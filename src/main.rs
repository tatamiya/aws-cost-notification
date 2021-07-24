mod cost_explorer;
mod cost_response_parser;
mod cost_usage_client;
mod date_range;
mod message_builder;
mod slack_notifier;
mod test_utils;

use chrono::Local;
use rusoto_core::Region;

use cost_explorer::CostExplorerService;
use cost_usage_client::CostAndUsageClient;
use date_range::ReportDateRange;
use message_builder::NotificationMessage;
use slack_notifier::SlackClient;

fn main() {
    println!("Start Program");
    let reporting_date = Local::today();
    let report_date_range = ReportDateRange::new(reporting_date);

    println!("Request Cost and Usage");
    let cost_usage_client = CostAndUsageClient::new(Region::ApNortheast1);
    let cost_explorer = CostExplorerService::new(cost_usage_client, report_date_range);
    println!("- Request Total Cost");
    let total_cost = cost_explorer.request_total_cost();
    println!("- Request Service Cost");
    let service_costs = cost_explorer.request_service_costs();

    println!("Build Notification Message");
    let notification_message = NotificationMessage::new(total_cost, service_costs);

    println!("Send Result to Slack");
    let slack_client = SlackClient::new();
    let res = slack_notifier::send_message_to_slack(slack_client, notification_message);
    match res {
        Ok(_) => println!("Notification Successfully Completed!"),
        Err(e) => panic!("Slack Notification Failed!: {}", e),
    }
}
