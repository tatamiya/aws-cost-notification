mod cost_explorer;
mod cost_response_parser;
mod cost_usage_client;
mod date_range;
mod message_builder;
mod slack_notifier;
mod test_utils;

use cost_explorer::CostExplorerService;
use cost_usage_client::{CostAndUsageClient, GetCostAndUsage};
use date_range::ReportDateRange;
use message_builder::NotificationMessage;
use slack_notifier::{PostToSlack, SlackClient};

use chrono::{Date, DateTime, Local, TimeZone};
use chrono_tz::Tz;
use dotenv::dotenv;
use lambda_runtime::{handler_fn, Context, Error};
use serde_json::Value;
use std::error;
use std::fmt::Display;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(lambda_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn lambda_handler(_: Value, _: Context) -> Result<(), Error> {
    let cost_usage_client = CostAndUsageClient::new();
    let slack_client = SlackClient::new();

    dotenv().ok();
    let tz_string = dotenv::var("REPORTING_TIMEZONE").expect("REPORTING_TIMEZONE not found");
    let now = Local::now();
    let reporting_date = date_in_specified_timezone(now, tz_string).unwrap();

    println!(
        "Launched lambda handler with reporting date {}",
        reporting_date
    );

    let res = request_cost_and_notify(cost_usage_client, slack_client, reporting_date).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string().into()),
    }
}

async fn request_cost_and_notify<C: GetCostAndUsage, S: PostToSlack, T>(
    cost_usage_client: C,
    slack_client: S,
    reporting_date: Date<T>,
) -> Result<(), Box<dyn error::Error>>
where
    T: TimeZone,
    <T as chrono::TimeZone>::Offset: Display,
{
    let report_date_range = ReportDateRange::new(reporting_date);

    let cost_explorer = CostExplorerService::new(cost_usage_client, report_date_range);
    let total_cost = cost_explorer.request_total_cost().await;
    let service_costs = cost_explorer.request_service_costs().await;

    let notification_message = NotificationMessage::new(total_cost, service_costs);

    let res = slack_notifier::send_message_to_slack(slack_client, notification_message);

    match res {
        Ok(_) => {
            println!("Notification Successfully Completed!");
            Ok(())
        }
        Err(e) => Err(format!("Slack Notification Failed!: {}", e).into()),
    }
}

fn date_in_specified_timezone<T: TimeZone>(
    datetime: DateTime<T>,
    tz_string: String,
) -> Result<Date<Tz>, Box<dyn error::Error>> {
    let timezone: Result<Tz, _> = tz_string.parse();
    match timezone {
        Ok(timezone) => Ok(datetime.with_timezone(&timezone).date()),
        Err(e) => Err(format!("Invalid Timezone!: {}", e).into()),
    }
}

#[cfg(test)]
mod test_reporting_date {
    use super::*;
    use chrono::{Local, TimeZone, Utc};

    #[test]
    fn convert_timezone_correctly() {
        let input_datetime = Local
            .datetime_from_str("2021-07-31 12:00:00 UTC", "%Y-%m-%d %H:%M:%S %Z")
            .unwrap();

        let tz_string = "Asia/Tokyo".to_string();

        let actual_date = date_in_specified_timezone(input_datetime, tz_string).unwrap();

        assert_eq!("2021-07-31JST", format!("{}", actual_date));
    }

    #[test]
    fn with_different_date() {
        let input_datetime = Utc
            .datetime_from_str("2021-07-31 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap();

        let tz_string = "Asia/Tokyo".to_string();

        let actual_date = date_in_specified_timezone(input_datetime, tz_string).unwrap();

        assert_eq!("2021-08-01JST", format!("{}", actual_date));
    }

    #[test]
    fn return_error_for_invalid_timezone() {
        let input_datetime = Local
            .datetime_from_str("2021-07-31 15:05:00 UTC", "%Y-%m-%d %H:%M:%S %Z")
            .unwrap();

        let tz_string = "Invalid/Timezone".to_string();

        let actual_date = date_in_specified_timezone(input_datetime, tz_string);

        assert!(actual_date.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::request_cost_and_notify;
    use crate::slack_notifier::PostToSlack;
    use crate::test_utils::{CostAndUsageClientStub, InputServiceCost};
    use chrono::{Local, TimeZone};
    use slack_hook::{Error, Payload};
    use tokio;

    struct SlackClientStub {
        fail: bool,
    }
    impl PostToSlack for SlackClientStub {
        fn post(self, _payload: &Payload) -> Result<(), Error> {
            if self.fail {
                Err(Error::from("Something Wrong!"))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn run_correctly() {
        let cost_usage_client_stub = CostAndUsageClientStub {
            service_costs: Some(vec![
                InputServiceCost::new("Amazon Simple Storage Service", "1234.56"),
                InputServiceCost::new("Amazon Elastic Compute Cloud", "31415.92"),
            ]),
            total_cost: Some(String::from("1234.56")),
        };

        let slack_client_stub = SlackClientStub { fail: false };

        let reporting_date = Local.ymd(2021, 8, 1);

        let res =
            request_cost_and_notify(cost_usage_client_stub, slack_client_stub, reporting_date)
                .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn return_error_when_slack_notification_fails() {
        let cost_usage_client_stub = CostAndUsageClientStub {
            service_costs: Some(vec![
                InputServiceCost::new("Amazon Simple Storage Service", "1234.56"),
                InputServiceCost::new("Amazon Elastic Compute Cloud", "31415.92"),
            ]),
            total_cost: Some(String::from("1234.56")),
        };

        let slack_client_stub = SlackClientStub { fail: true };

        let reporting_date = Local.ymd(2021, 8, 1);

        let res =
            request_cost_and_notify(cost_usage_client_stub, slack_client_stub, reporting_date)
                .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    #[should_panic]
    async fn panic_when_total_cost_is_empty() {
        let cost_usage_client_stub = CostAndUsageClientStub {
            service_costs: Some(vec![
                InputServiceCost::new("Amazon Simple Storage Service", "1234.56"),
                InputServiceCost::new("Amazon Elastic Compute Cloud", "31415.92"),
            ]),
            total_cost: None,
        };

        let slack_client_stub = SlackClientStub { fail: false };

        let reporting_date = Local.ymd(2021, 8, 1);

        let _res =
            request_cost_and_notify(cost_usage_client_stub, slack_client_stub, reporting_date)
                .await;
    }

    #[tokio::test]
    #[should_panic]
    async fn panic_when_service_costs_is_empty() {
        let cost_usage_client_stub = CostAndUsageClientStub {
            service_costs: None,
            total_cost: Some(String::from("1234.56")),
        };

        let slack_client_stub = SlackClientStub { fail: false };

        let reporting_date = Local.ymd(2021, 8, 1);

        let _res =
            request_cost_and_notify(cost_usage_client_stub, slack_client_stub, reporting_date)
                .await;
    }
}
