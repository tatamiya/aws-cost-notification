//! # AWS Cost Notifier
//!
//! A Lambda function to retrieve AWS costs from Cost Explorer
//! and notify them to Slack.

/// Call AWS CostExplorer API and retrieve total cost and costs for each service.
mod cost_explorer;
/// Build notification message from API responses
mod message_builder;
/// Set the period to retrieve the AWS costs.
mod reporting_date;
/// Send a message to notify the AWS costs to Slack.
mod slack_notifier;

use cost_explorer::cost_usage_client::{CostAndUsageClient, GetCostAndUsage};
use cost_explorer::CostExplorerService;
use message_builder::NotificationMessage;
use reporting_date::{date_in_specified_timezone, ReportDateRange};
use slack_notifier::{SendMessage, SlackNotifier};

use chrono::{Date, Local, TimeZone};
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
    let slack_notifier = SlackNotifier::new();

    dotenv().ok();
    let tz_string = dotenv::var("REPORTING_TIMEZONE").expect("REPORTING_TIMEZONE not found");
    let now = Local::now();
    let reporting_date = date_in_specified_timezone(now, tz_string).unwrap();

    println!(
        "Launched lambda handler with reporting date {}",
        reporting_date
    );

    let res = request_cost_and_notify(cost_usage_client, slack_notifier, reporting_date).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string().into()),
    }
}

/// The core function of the whole process.
/// `cost_usage_client` retrieves AWS costs via CostExplorer API
/// and `notifier` sends a message to Slack.
///
/// The period of the cost aggregation is from the first date
/// of the month upto the `reporting_date`.
/// If the `reporting_date` is the first date of the month,
/// the start date is set to the first date of the previous month.
///
/// You can execute integration tests by using client stubs and designating
/// the reporting date.
async fn request_cost_and_notify<C: GetCostAndUsage, N: SendMessage, T>(
    cost_usage_client: C,
    notifier: N,
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

    let res = notifier.send(notification_message);

    match res {
        Ok(_) => {
            println!("Notification Successfully Completed!");
            Ok(())
        }
        Err(e) => Err(format!("Slack Notification Failed!: {}", e).into()),
    }
}

#[cfg(test)]
mod integration_tests {
    use super::request_cost_and_notify;
    use crate::cost_explorer::test_utils::{CostAndUsageClientStub, InputServiceCost};
    use crate::message_builder::NotificationMessage;
    use crate::slack_notifier::SendMessage;
    use chrono::{Local, TimeZone};
    use slack_hook::Error;
    use tokio;

    struct SlackNotifierStub {
        fail: bool,
    }
    impl SendMessage for SlackNotifierStub {
        fn send(self, _message: NotificationMessage) -> Result<(), Error> {
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

        let slack_client_stub = SlackNotifierStub { fail: false };

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

        let slack_client_stub = SlackNotifierStub { fail: true };

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

        let slack_client_stub = SlackNotifierStub { fail: false };

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

        let slack_client_stub = SlackNotifierStub { fail: false };

        let reporting_date = Local.ymd(2021, 8, 1);

        let _res =
            request_cost_and_notify(cost_usage_client_stub, slack_client_stub, reporting_date)
                .await;
    }
}
