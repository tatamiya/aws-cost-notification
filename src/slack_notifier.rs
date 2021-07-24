use serde_json::{json, Value as JsonValue};

use crate::message_builder::NotificationMessage;

impl NotificationMessage {
    fn as_payload(&self) -> JsonValue {
        json!(
            {
                "blocks": [
                    {
                        "type": "header",
                        "text": {
                            "type": "plain_text",
                            "text": self.header
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
                                "text": self.body
                            }
                        ]
                    }
                ]
            }
        )
    }
}

use dotenv::dotenv;
use futures::executor::block_on;
use std::result::Result;

// extern crate slack_hook;

use reqwest;
//use tokio_compat_02::FutureExt;
// use slack_hook::{Error, PayloadBuilder, Slack};

pub fn send_message_to_slack(
    message: JsonValue,
    channel: &str,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    dotenv().ok();
    let webhook_url = dotenv::var("SLACK_WEBHOOK_URL").expect("Webhook URL not found.");
    let client = reqwest::blocking::Client::new();
    let res = client.post(webhook_url).json(&message).send();
    //    let slack = Slack::new(webhook_url.as_ref()).unwrap();
    //    let p = PayloadBuilder::new()
    //        .text(message.to_string())
    //        .channel(channel)
    //        .build()
    //        .unwrap();
    //
    //    slack.send(&p)
    res
}

#[cfg(test)]
mod test_build_payload {
    use crate::message_builder::NotificationMessage;
    use serde_json::json;

    #[test]
    fn build_payload_correctly() {
        let sample_message = NotificationMessage {
            header: "07/01~07/11の請求額は、1.62 USDです。".to_string(),
            body: "・AWS CloudTrail: 0.01 USD\n・AWS Cost Explorer: 0.18 USD".to_string(),
        };

        let expected_payload = json!(
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

        let actual_payload = sample_message.as_payload();

        assert_eq!(expected_payload, actual_payload);
    }
}
