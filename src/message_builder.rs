use crate::cost_explorer::{ParsedServiceCost, ParsedTotalCost};

impl ParsedServiceCost {
    fn to_message_line(&self) -> String {
        format!("・{}: {}", self.service_name, self.cost)
    }
}

impl ParsedTotalCost {
    fn to_message_header(&self) -> String {
        format!("{}の請求額は、{}です。", self.date_range, self.cost)
    }
}

struct NotificationMessage {
    header: String,
    body: String,
}
impl NotificationMessage {
    fn new(total_cost: ParsedTotalCost, service_costs: Vec<ParsedServiceCost>) -> Self {
        NotificationMessage {
            header: total_cost.to_message_header(),
            body: service_costs
                .iter()
                .map(|x| x.to_message_line())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[cfg(test)]
mod test_build_message {
    use super::*;
    use crate::cost_explorer::{Cost, ReportedDateRange};
    use chrono::{Local, TimeZone};

    #[test]
    fn convert_total_cost_into_message_header_correctly() {
        let sample_total_cost = ParsedTotalCost {
            date_range: ReportedDateRange {
                start_date: Local.ymd(2021, 7, 1),
                end_date: Local.ymd(2021, 7, 11),
            },
            cost: Cost {
                amount: 1.6234,
                unit: "USD".to_string(),
            },
        };
        let expected_header = "07/01~07/11の請求額は、1.62 USDです。";
        let actual_header = sample_total_cost.to_message_header();

        assert_eq!(expected_header, actual_header);
    }
    #[test]
    fn convert_service_cost_into_message_line_correctly() {
        let sample_service_cost = ParsedServiceCost {
            service_name: "AWS CloudTrail".to_string(),
            cost: Cost {
                amount: 0.0123,
                unit: "USD".to_string(),
            },
        };
        let expected_line = "・AWS CloudTrail: 0.01 USD";
        let actual_line = sample_service_cost.to_message_line();

        assert_eq!(expected_line, actual_line);
    }

    #[test]
    fn construct_notification_message_correctly() {
        let sample_total_cost = ParsedTotalCost {
            date_range: ReportedDateRange {
                start_date: Local.ymd(2021, 7, 1),
                end_date: Local.ymd(2021, 7, 11),
            },
            cost: Cost {
                amount: 1.6234,
                unit: "USD".to_string(),
            },
        };

        let sample_service_costs = vec![
            ParsedServiceCost {
                service_name: "AWS CloudTrail".to_string(),
                cost: Cost {
                    amount: 0.0123,
                    unit: "USD".to_string(),
                },
            },
            ParsedServiceCost {
                service_name: "AWS Cost Explorer".to_string(),
                cost: Cost {
                    amount: 0.182345,
                    unit: "USD".to_string(),
                },
            },
        ];

        let actual_message = NotificationMessage::new(sample_total_cost, sample_service_costs);

        assert_eq!(
            "07/01~07/11の請求額は、1.62 USDです。",
            actual_message.header,
        );

        assert_eq!(
            "・AWS CloudTrail: 0.01 USD\n・AWS Cost Explorer: 0.18 USD",
            actual_message.body,
        );
    }
}
