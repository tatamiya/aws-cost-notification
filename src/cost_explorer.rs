use chrono::{Date, Local, NaiveDate, TimeZone};
use futures::executor::block_on;
use rusoto_ce::{GetCostAndUsageRequest, GetCostAndUsageResponse, Group, GroupDefinition};

use crate::cost_usage_client::GetCostAndUsage;
use crate::date_range::ReportDateRange;

struct CostExplorerService<T: GetCostAndUsage> {
    client: T,
    report_date_range: ReportDateRange,
}
impl<T: GetCostAndUsage> CostExplorerService<T> {
    fn new(client: T, report_date_range: ReportDateRange) -> Self {
        CostExplorerService {
            client: client,
            report_date_range: report_date_range,
        }
    }

    fn request_total_cost(self) -> ParsedTotalCost {
        let request: GetCostAndUsageRequest =
            build_cost_and_usage_request(self.report_date_range, true);

        let res = block_on(self.client.get_cost_and_usage(request)).unwrap();
        ParsedTotalCost::from_response(&res)
    }

    fn request_service_costs(self) -> Vec<ParsedServiceCost> {
        let request: GetCostAndUsageRequest =
            build_cost_and_usage_request(self.report_date_range, false);
        let res = block_on(self.client.get_cost_and_usage(request)).unwrap();
        ParsedServiceCost::from_response(&res)
    }
}

fn build_cost_and_usage_request(
    report_date_range: ReportDateRange,
    is_total: bool,
) -> GetCostAndUsageRequest {
    let group_by: Option<Vec<GroupDefinition>> = match is_total {
        true => None,
        false => Some(vec![GroupDefinition {
            type_: Some("DIMENSION".to_string()),
            key: Some("SERVICE".to_string()),
        }]),
    };
    GetCostAndUsageRequest {
        filter: None,
        granularity: "MONTHLY".to_string(),
        group_by: group_by,
        metrics: vec!["AmortizedCost".to_string()],
        next_page_token: None,
        time_period: report_date_range.as_date_interval(),
    }
}

#[derive(Debug, PartialEq)]
struct ParsedTotalCost {
    start_date: Date<Local>,
    end_date: Date<Local>,
    cost: f32,
    unit: String,
}
impl ParsedTotalCost {
    fn from_response(res: &GetCostAndUsageResponse) -> Self {
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
}

#[derive(Debug, PartialEq)]
struct ParsedServiceCost {
    service_name: String,
    cost: f32,
    unit: String,
}
impl ParsedServiceCost {
    fn from_group(group: &Group) -> Self {
        let service_name = &group.keys.as_ref().unwrap()[0];
        let amortized_cost = group
            .metrics
            .as_ref()
            .unwrap()
            .get("AmortizedCost")
            .unwrap();
        let cost = amortized_cost
            .amount
            .as_ref()
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let unit = amortized_cost.unit.as_ref().unwrap().to_string();
        ParsedServiceCost {
            service_name: service_name.to_string(),
            cost: cost,
            unit: unit,
        }
    }
    fn from_response(res: &GetCostAndUsageResponse) -> Vec<Self> {
        let result_by_time = &res.results_by_time.as_ref().unwrap()[0];
        let groups = result_by_time.groups.as_ref().unwrap();
        groups
            .iter()
            .map(|x| ParsedServiceCost::from_group(&x))
            .collect()
    }
}

fn parse_timestamp_into_local_date(timestamp: &str) -> chrono::LocalResult<Date<Local>> {
    let parsed_start_date = NaiveDate::parse_from_str(timestamp, "%Y-%m-%d")
        .ok()
        .unwrap();
    Local.from_local_date(&parsed_start_date)
}

mod test_helpers {
    use super::GetCostAndUsage;
    use async_trait::async_trait;
    use rusoto_ce::*;
    use rusoto_core::RusotoError;
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct InputServiceCost {
        service_name: String,
        cost: String,
    }
    impl InputServiceCost {
        pub fn new(service_name: &str, cost: &str) -> Self {
            InputServiceCost {
                service_name: String::from(service_name),
                cost: String::from(cost),
            }
        }

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

    pub fn prepare_sample_response(
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

    pub struct CostAndUsageClientStub {
        pub service_costs: Option<Vec<InputServiceCost>>,
        pub total_cost: Option<String>,
    }
    #[async_trait]
    impl GetCostAndUsage for CostAndUsageClientStub {
        async fn get_cost_and_usage(
            &self,
            input: GetCostAndUsageRequest,
        ) -> Result<GetCostAndUsageResponse, RusotoError<GetCostAndUsageError>> {
            let service_costs: Option<Vec<InputServiceCost>>;
            let total_cost: Option<String>;
            match input.group_by {
                Some(_) => {
                    service_costs = self.service_costs.clone();
                    total_cost = None;
                }
                None => {
                    service_costs = None;
                    total_cost = self.total_cost.clone();
                }
            }
            let response: GetCostAndUsageResponse =
                prepare_sample_response(Some(input.time_period), total_cost, service_costs);
            Ok(response)
        }
    }
}

#[cfg(test)]
mod test_cost_explorer_service {
    use super::test_helpers::*;
    use super::*;
    use crate::date_range::ReportDateRange;
    use chrono::{Local, TimeZone};

    #[test]
    fn request_total_cost_correctly() {
        let client_stub = CostAndUsageClientStub {
            service_costs: None,
            total_cost: Some(String::from("1234.56")),
        };
        let report_date_range = ReportDateRange::new(Local.ymd(2021, 7, 23));
        let explorer = CostExplorerService::new(client_stub, report_date_range);

        let expected_total_cost = ParsedTotalCost {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 23),
            cost: 1234.56,
            unit: String::from("USD"),
        };

        let actual_total_cost = explorer.request_total_cost();

        assert_eq!(expected_total_cost, actual_total_cost);
    }

    #[test]
    fn request_service_costs_correctly() {
        let client_stub = CostAndUsageClientStub {
            service_costs: Some(vec![
                InputServiceCost::new("Amazon Simple Storage Service", "1234.56"),
                InputServiceCost::new("Amazon Elastic Compute Cloud", "31415.92"),
            ]),
            total_cost: None,
        };
        let report_date_range = ReportDateRange::new(Local.ymd(2021, 7, 23));
        let explorer = CostExplorerService::new(client_stub, report_date_range);

        let expected_service_costs = vec![
            ParsedServiceCost {
                service_name: String::from("Amazon Simple Storage Service"),
                cost: 1234.56,
                unit: String::from("USD"),
            },
            ParsedServiceCost {
                service_name: String::from("Amazon Elastic Compute Cloud"),
                cost: 31415.92,
                unit: String::from("USD"),
            },
        ];

        let actual_service_costs = explorer.request_service_costs();

        assert_eq!(expected_service_costs, actual_service_costs);
    }
}

#[cfg(test)]
mod test_build_request {
    use super::*;
    use crate::date_range::ReportDateRange;
    use chrono::{Local, TimeZone};
    use rusoto_ce::DateInterval;

    #[test]
    fn build_total_cost_request_correctly() {
        let input_date_range = ReportDateRange::new(Local.ymd(2021, 7, 23));
        let expected_request = GetCostAndUsageRequest {
            filter: None,
            granularity: String::from("MONTHLY"),
            group_by: None,
            metrics: vec![String::from("AmortizedCost")],
            next_page_token: None,
            time_period: DateInterval {
                start: "2021-07-01".to_string(),
                end: "2021-07-23".to_string(),
            },
        };
        let actual_request = build_cost_and_usage_request(input_date_range, true);
        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn build_service_costs_request_correctly() {
        let input_date_range = ReportDateRange::new(Local.ymd(2021, 7, 23));
        let expected_request = GetCostAndUsageRequest {
            filter: None,
            granularity: String::from("MONTHLY"),
            group_by: Some(vec![GroupDefinition {
                type_: Some("DIMENSION".to_string()),
                key: Some("SERVICE".to_string()),
            }]),
            metrics: vec![String::from("AmortizedCost")],
            next_page_token: None,
            time_period: DateInterval {
                start: "2021-07-01".to_string(),
                end: "2021-07-23".to_string(),
            },
        };
        let actual_request = build_cost_and_usage_request(input_date_range, false);

        assert_eq!(expected_request, actual_request);
    }
}

#[cfg(test)]
mod test_parsers {
    use super::test_helpers::*;
    use super::*;
    use rusoto_ce::*;

    #[test]
    fn parse_timestamp_into_local_date_correctly() {
        let input_timestamp = "2021-07-22";
        let expected_parsed_date = Local.ymd(2021, 7, 22);

        let actual_parsed_date = parse_timestamp_into_local_date(input_timestamp).unwrap();
        assert_eq!(expected_parsed_date, actual_parsed_date);
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

        let actual_parsed_total_cost = ParsedTotalCost::from_response(&input_response);

        assert_eq!(expected_parsed_total_cost, actual_parsed_total_cost);
    }

    #[test]
    fn parse_service_costs_correctly() {
        let input_response: GetCostAndUsageResponse = prepare_sample_response(
            None,
            None,
            Some(vec![
                InputServiceCost::new("Amazon Simple Storage Service", "1234.56"),
                InputServiceCost::new("Amazon Elastic Compute Cloud", "31415.92"),
            ]),
        );
        let expected_parsed_service_costs = vec![
            ParsedServiceCost {
                service_name: String::from("Amazon Simple Storage Service"),
                cost: 1234.56,
                unit: String::from("USD"),
            },
            ParsedServiceCost {
                service_name: String::from("Amazon Elastic Compute Cloud"),
                cost: 31415.92,
                unit: String::from("USD"),
            },
        ];
        let actual_parsed_service_costs = ParsedServiceCost::from_response(&input_response);

        assert_eq!(expected_parsed_service_costs, actual_parsed_service_costs);
    }
}
