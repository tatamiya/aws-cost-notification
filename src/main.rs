mod cost_explorer;
mod cost_response_parser;
mod cost_usage_client;
mod date_range;
mod message_builder;
mod slack_notifier;
mod test_utils;

use message_builder::NotificationMessage;
use slack_notifier::SlackClient;

fn main() {
    let slack_client = SlackClient::new();
    let sample_message = NotificationMessage {
        header: "07/01~07/11の請求額は、1.62 USDです。".to_string(),
        body: "・AWS CloudTrail: 0.01 USD\n・AWS Cost Explorer: 0.18 USD".to_string(),
    };
    let res = slack_notifier::send_message_to_slack(slack_client, sample_message);
    match res {
        Ok(_) => println!("Notification Successfully Completed!"),
        Err(e) => panic!("Slack Notification Failed!: {}", e),
    }
}
