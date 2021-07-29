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

use chrono::{Date, Local, TimeZone};
use chrono_tz::Tz;
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

    let timezone: Tz = "Asia/Tokyo".parse().unwrap();
    let now = Local::now();
    let reporting_date = now.with_timezone(&timezone).date();

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
