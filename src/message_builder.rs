use crate::cost_response_parser::{Cost, ReportedDateRange, ServiceCost, TotalCost};
use chrono::Datelike;
use std::fmt;

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.amount, self.unit)
    }
}

impl fmt::Display for ReportedDateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}/{:02}~{:02}/{:02}",
            self.start_date.month(),
            self.start_date.day(),
            self.end_date.month(),
            self.end_date.day(),
        )
    }
}

impl ServiceCost {
    fn to_message_line(&self) -> String {
        format!("・{}: {}", self.service_name, self.cost)
    }
}

impl TotalCost {
    fn to_message_header(&self) -> String {
        format!("{}の請求額は、{}です。", self.date_range, self.cost)
    }
}

pub struct NotificationMessage {
    pub header: String,
    pub body: String,
}
impl NotificationMessage {
    pub fn new(total_cost: TotalCost, service_costs: Vec<ServiceCost>) -> Self {
        // TODO: There must be more smart way to copy and sort a vector.
        let mut sorted_service_costs = service_costs.iter().collect::<Vec<_>>();
        sorted_service_costs.sort_by(|a, b| b.cost.amount.partial_cmp(&a.cost.amount).unwrap());

        NotificationMessage {
            header: total_cost.to_message_header(),
            body: sorted_service_costs
                .iter()
                .filter(|x| format!("{}", x.cost) != "0.00 USD")
                .map(|x| x.to_message_line())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[cfg(test)]
mod test_cost_representation {
    use crate::cost_response_parser::Cost;

    #[test]
    fn display_correctly() {
        let input_cost = Cost {
            amount: 132.2345,
            unit: "USD".to_string(),
        };
        assert_eq!("132.23 USD", format!("{}", input_cost));
    }
}

#[cfg(test)]
mod test_date_range_representation {
    use crate::cost_response_parser::ReportedDateRange;
    use chrono::{Local, TimeZone};

    #[test]
    fn test_display_correctly() {
        let sample_date_range = ReportedDateRange {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 23),
        };
        assert_eq!("07/01~07/23", format!("{}", sample_date_range))
    }
}
#[cfg(test)]
mod test_build_message {
    use super::*;
    use crate::cost_response_parser::{Cost, ReportedDateRange};
    use chrono::{Local, TimeZone};

    #[test]
    fn convert_total_cost_into_message_header_correctly() {
        let sample_total_cost = TotalCost {
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
        let sample_service_cost = ServiceCost {
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
        let sample_total_cost = TotalCost {
            date_range: ReportedDateRange {
                start_date: Local.ymd(2021, 7, 1),
                end_date: Local.ymd(2021, 7, 11),
            },
            cost: Cost {
                amount: 1.357,
                unit: "USD".to_string(),
            },
        };

        let sample_service_costs = vec![
            ServiceCost {
                service_name: "AWS CloudTrail".to_string(),
                cost: Cost {
                    amount: 1.234,
                    unit: "USD".to_string(),
                },
            },
            ServiceCost {
                service_name: "AWS Cost Explorer".to_string(),
                cost: Cost {
                    amount: 0.123,
                    unit: "USD".to_string(),
                },
            },
        ];

        let actual_message = NotificationMessage::new(sample_total_cost, sample_service_costs);

        assert_eq!(
            "07/01~07/11の請求額は、1.36 USDです。",
            actual_message.header,
        );

        assert_eq!(
            "・AWS CloudTrail: 1.23 USD\n・AWS Cost Explorer: 0.12 USD",
            actual_message.body,
        );
    }

    #[test]
    fn sort_service_costs_by_descending_order_correctly() {
        let sample_total_cost = TotalCost {
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
            ServiceCost {
                service_name: "AWS CloudTrail".to_string(),
                cost: Cost {
                    amount: 0.0123,
                    unit: "USD".to_string(),
                },
            },
            ServiceCost {
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
            "・AWS Cost Explorer: 0.18 USD\n・AWS CloudTrail: 0.01 USD",
            actual_message.body,
        );
    }

    #[test]
    fn message_line_is_not_displayed_when_cost_is_zero() {
        let sample_total_cost = TotalCost {
            date_range: ReportedDateRange {
                start_date: Local.ymd(2021, 7, 1),
                end_date: Local.ymd(2021, 7, 11),
            },
            cost: Cost {
                amount: 0.01,
                unit: "USD".to_string(),
            },
        };

        let sample_service_costs = vec![
            ServiceCost {
                service_name: "AWS CloudTrail".to_string(),
                cost: Cost {
                    amount: 0.01,
                    unit: "USD".to_string(),
                },
            },
            ServiceCost {
                service_name: "AWS Cost Explorer".to_string(),
                cost: Cost {
                    amount: 0.001,
                    unit: "USD".to_string(),
                },
            },
            ServiceCost {
                service_name: "AWS Dummy Service".to_string(),
                cost: Cost {
                    amount: 0.005,
                    unit: "USD".to_string(),
                },
            },
        ];

        let actual_message = NotificationMessage::new(sample_total_cost, sample_service_costs);

        assert_eq!(
            "07/01~07/11の請求額は、0.01 USDです。",
            actual_message.header,
        );

        assert_eq!("・AWS CloudTrail: 0.01 USD", actual_message.body,);
    }
}
