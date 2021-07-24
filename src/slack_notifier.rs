use serde_json::{json, Value as JsonValue};

use crate::message_builder::NotificationMessage;

use dotenv::dotenv;
use std::result::Result;

extern crate slack_hook;

use slack_hook::{Attachment, Error, HexColor, Payload, PayloadBuilder, Slack, SlackText, TryFrom};

pub struct SlackClient {
    slack: Slack,
}
impl SlackClient {
    pub fn new() -> Self {
        dotenv().ok();
        let webhook_url = dotenv::var("SLACK_WEBHOOK_URL").expect("Webhook URL not found.");
        let slack = Slack::new(webhook_url.as_ref()).unwrap();
        SlackClient { slack: slack }
    }
    fn post(self, payload: &Payload) -> Result<(), Error> {
        self.slack.send(&payload)
    }
}

pub fn send_message_to_slack(
    client: SlackClient,
    message: NotificationMessage,
) -> Result<(), Error> {
    dotenv().ok();
    let payload = PayloadBuilder::new()
        .attachments(vec![Attachment {
            pretext: Some(SlackText::new(message.header)),
            text: Some(SlackText::new(message.body)),
            color: Some(HexColor::try_from("#36a64f").unwrap()),
            ..Attachment::default()
        }])
        .build()
        .unwrap();
    client.post(&payload)
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

        //let actual_payload = sample_message.as_payload();

        //assert_eq!(expected_payload, actual_payload);
    }
}
