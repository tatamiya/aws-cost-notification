use chrono::{Date, Local, NaiveDate, TimeZone};
use rusoto_ce::GetCostAndUsageResponse;

#[derive(Debug, PartialEq)]
struct ParsedTotalCost {
    start_date: Date<Local>,
    end_date: Date<Local>,
    cost: f32,
    unit: String,
}

#[derive(Debug, PartialEq)]
struct ParsedServiceCost {
    service_name: String,
    cost: f32,
    unit: String,
}

fn parse_total_cost(res: GetCostAndUsageResponse) -> ParsedTotalCost {
    let result_by_time = &res.results_by_time.as_ref().unwrap()[0];
    let time_period = result_by_time.time_period.as_ref().unwrap();

    let parsed_start_date = parse_timestamp_into_local_date(&time_period.start).unwrap();
    let parsed_end_date = parse_timestamp_into_local_date(&time_period.end).unwrap();

    let amortized_cost = result_by_time
        .total
        .as_ref()
        .unwrap()
        .get("AmortizedCost")
        .unwrap();

    let parsed_cost = amortized_cost
        .amount
        .as_ref()
        .unwrap()
        .parse::<f32>()
        .unwrap();

    let parsed_cost_unit = amortized_cost.unit.as_ref().unwrap().to_string();

    ParsedTotalCost {
        start_date: parsed_start_date,
        end_date: parsed_end_date,
        cost: parsed_cost,
        unit: parsed_cost_unit,
    }
}

fn parse_timestamp_into_local_date(timestamp: &str) -> chrono::LocalResult<Date<Local>> {
    let parsed_start_date = NaiveDate::parse_from_str(timestamp, "%Y-%m-%d")
        .ok()
        .unwrap();
    Local.from_local_date(&parsed_start_date)
}

fn parse_service_costs(res: GetCostAndUsageResponse) -> Vec<ParsedServiceCost> {
    vec![ParsedServiceCost {
        service_name: String::from("Amazon Simple Storage Service"),
        cost: 1234.56,
        unit: String::from("USD"),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_ce::*;
    use std::collections::HashMap;

    #[test]
    fn parse_timestamp_into_local_date_correctly() {
        let input_timestamp = "2021-07-22";
        let expected_parsed_date = Local.ymd(2021, 7, 22);

        let actual_parsed_date = parse_timestamp_into_local_date(input_timestamp).unwrap();
        assert_eq!(expected_parsed_date, actual_parsed_date);
    }

    struct InputServiceCost {
        service_name: String,
        cost: String,
    }
    impl InputServiceCost {
        fn as_group(&self) -> Group {
            let mut metrics = HashMap::new();
            metrics.insert(
                String::from("AmortizedCost"),
                MetricValue {
                    amount: Some(self.cost.clone()),
                    unit: Some(String::from("USD")),
                },
            );
            Group {
                keys: Some(vec![self.service_name.clone()]),
                metrics: Some(metrics),
            }
        }
    }

    fn prepare_sample_response(
        date_interval: Option<DateInterval>,
        total_cost: Option<String>,
        service_costs: Option<Vec<InputServiceCost>>,
    ) -> GetCostAndUsageResponse {
        let mut total = HashMap::new();
        total.insert(
            String::from("AmortizedCost"),
            MetricValue {
                amount: total_cost,
                unit: Some(String::from("USD")),
            },
        );
        let input_grouped_costs: Option<Vec<Group>> = match service_costs {
            Some(service_costs) => Some(service_costs.iter().map(|x| x.as_group()).collect()),
            None => None,
        };

        GetCostAndUsageResponse {
            dimension_value_attributes: None,
            group_definitions: None,
            next_page_token: None,
            results_by_time: Some(vec![ResultByTime {
                estimated: Some(false),
                groups: input_grouped_costs,
                time_period: date_interval,
                total: Some(total),
            }]),
        }
    }

    #[test]
    fn parse_total_cost_correctly() {
        let input_response: GetCostAndUsageResponse = prepare_sample_response(
            Some(DateInterval {
                start: String::from("2021-07-01"),
                end: String::from("2021-07-18"),
            }),
            Some(String::from("1234.56")),
            None,
        );

        let expected_parsed_total_cost = ParsedTotalCost {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 18),
            cost: 1234.56,
            unit: String::from("USD"),
        };

        let actual_parsed_total_cost = parse_total_cost(input_response);

        assert_eq!(expected_parsed_total_cost, actual_parsed_total_cost);
    }

    #[test]
    fn parse_service_costs_correctly() {
        let input_response: GetCostAndUsageResponse = prepare_sample_response(
            None,
            None,
            Some(vec![InputServiceCost {
                service_name: String::from("Amazon Simple Storage Service"),
                cost: String::from("1234.56"),
            }]),
        );
        let expected_parsed_service_costs = vec![ParsedServiceCost {
            service_name: String::from("Amazon Simple Storage Service"),
            cost: 1234.56,
            unit: String::from("USD"),
        }];
        let actual_parsed_service_costs = parse_service_costs(input_response);

        assert_eq!(expected_parsed_service_costs, actual_parsed_service_costs);
    }
}
