use crate::message_builder::NotificationMessage;

use dotenv::dotenv;
use std::result::Result;

extern crate slack_hook;

use slack_hook::{Attachment, Error, HexColor, Payload, PayloadBuilder, Slack, SlackText, TryFrom};

impl NotificationMessage {
    fn as_attachment(self, color: &str) -> Attachment {
        Attachment {
            pretext: Some(SlackText::new(self.header)),
            text: Some(SlackText::new(self.body)),
            color: Some(HexColor::try_from(color).unwrap()),
            ..Attachment::default()
        }
    }
}

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
    let payload = PayloadBuilder::new()
        .attachments(vec![message.as_attachment("#36a64f")])
        .build()
        .unwrap();

    client.post(&payload)
}

#[cfg(test)]
mod test_build_attachment {
    use crate::message_builder::NotificationMessage;
    use slack_hook::{Attachment, HexColor, SlackText, TryFrom};

    #[test]
    fn build_attachment_correctly() {
        let sample_message = NotificationMessage {
            header: "07/01~07/11の請求額は、1.62 USDです。".to_string(),
            body: "・AWS CloudTrail: 0.01 USD\n・AWS Cost Explorer: 0.18 USD".to_string(),
        };

        let expected_attchment = Attachment {
            pretext: Some(SlackText::new("07/01~07/11の請求額は、1.62 USDです。")),
            text: Some(SlackText::new(
                "・AWS CloudTrail: 0.01 USD\n・AWS Cost Explorer: 0.18 USD",
            )),
            color: Some(HexColor::try_from("#36a64f").unwrap()),
            ..Attachment::default()
        };
        let actual_attachment = sample_message.as_attachment("#36a64f");

        assert_eq!(expected_attchment, actual_attachment);
    }
}
