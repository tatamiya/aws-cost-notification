use chrono::{Date, Local, TimeZone};
use rusoto_ce::GetCostAndUsageResponse;

#[derive(Debug, PartialEq)]
struct ParsedTotalCost {
    start_date: Date<Local>,
    end_date: Date<Local>,
    cost: f32,
    unit: String,
}

fn parse_total_cost(res: GetCostAndUsageResponse) -> ParsedTotalCost {
    ParsedTotalCost {
        start_date: Local.ymd(2021, 7, 1),
        end_date: Local.ymd(2021, 7, 18),
        cost: 1234.56,
        unit: String::from("USD"),
    }
}

#[cfg(test)]
mod total_cost_tests {
    use super::*;
    use rusoto_ce::*;
    use std::collections::HashMap;

    #[test]
    fn parse_response_correctly() {
        let mut input_total_cost = HashMap::new();
        input_total_cost.insert(
            String::from("AmortizedCost"),
            MetricValue {
                amount: Some(String::from("1234.56")),
                unit: Some(String::from("USD")),
            },
        );
        let input_response = GetCostAndUsageResponse {
            dimension_value_attributes: None,
            group_definitions: None,
            next_page_token: None,
            results_by_time: Some(vec![ResultByTime {
                estimated: Some(false),
                groups: None,
                time_period: Some(DateInterval {
                    start: String::from("2021-07-01"),
                    end: String::from("2021-07-18"),
                }),
                total: Some(input_total_cost),
            }]),
        };

        let expected_parsed_total_cost = ParsedTotalCost {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 18),
            cost: 1234.56,
            unit: String::from("USD"),
        };

        let actual_parsed_total_cost = parse_total_cost(input_response);

        assert_eq!(expected_parsed_total_cost, actual_parsed_total_cost);
    }
}
