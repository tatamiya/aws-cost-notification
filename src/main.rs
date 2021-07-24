mod cost_explorer;
mod cost_response_parser;
mod cost_usage_client;
mod date_range;
mod message_builder;
mod slack_notifier;
mod test_utils;

use serde_json::json;

fn main() {
    let channel = "#notification-test";
    let text = json!(
        {
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": "07/01~07/11の請求額は、1.62 USDです。"
                    }
                },
                {
                    "type": "divider"
                },
                {
                    "type": "section",
                    "fields": [
                        {
                            "type": "mrkdwn",
                            "text": "・AWS CloudTrail: 0.01 USD\n・AWS Cost Explorer: 0.18 USD"
                        }
                    ]
                }
            ]
        }
    );

    let res = slack_notifier::send_message_to_slack(text, channel);
    match res {
        Ok(_) => println!("Notification Successfully Completed!"),
        Err(e) => panic!("Slack Notification Failed!: {}", e),
    }
}
