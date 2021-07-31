mod cost_explorer;
mod cost_response_parser;
mod cost_usage_client;
mod date_range;
mod message_builder;
mod slack_notifier;
mod test_utils;

use cost_explorer::CostExplorerService;
use cost_usage_client::CostAndUsageClient;
use date_range::ReportDateRange;
use message_builder::NotificationMessage;
use slack_notifier::SlackClient;

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

async fn request_cost_and_notify<T>(
    cost_usage_client: CostAndUsageClient,
    slack_client: SlackClient,
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
