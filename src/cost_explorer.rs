pub mod cost_response_parser;
pub mod cost_usage_client;
pub mod test_utils;

use chrono::TimeZone;
use rusoto_ce::{GetCostAndUsageRequest, GroupDefinition};
use std::fmt::Display;

use crate::reporting_date::ReportDateRange;
use cost_response_parser::{ServiceCost, TotalCost};
use cost_usage_client::GetCostAndUsage;

pub struct CostExplorerService<T: GetCostAndUsage, U>
where
    U: TimeZone,
    <U as chrono::TimeZone>::Offset: Display,
{
    client: T,
    report_date_range: ReportDateRange<U>,
}
impl<T: GetCostAndUsage, U> CostExplorerService<T, U>
where
    U: TimeZone,
    <U as chrono::TimeZone>::Offset: Display,
{
    pub fn new(client: T, report_date_range: ReportDateRange<U>) -> Self {
        CostExplorerService {
            client: client,
            report_date_range: report_date_range,
        }
    }

    pub async fn request_total_cost(&self) -> TotalCost {
        let request: GetCostAndUsageRequest =
            build_cost_and_usage_request(&self.report_date_range, true);

        let res = self.client.get_cost_and_usage(request).await.unwrap();
        res.into()
    }

    pub async fn request_service_costs(&self) -> Vec<ServiceCost> {
        let request: GetCostAndUsageRequest =
            build_cost_and_usage_request(&self.report_date_range, false);
        let res = self.client.get_cost_and_usage(request).await.unwrap();
        ServiceCost::from_response(&res)
    }
}

fn build_cost_and_usage_request<U>(
    report_date_range: &ReportDateRange<U>,
    is_total: bool,
) -> GetCostAndUsageRequest
where
    U: TimeZone,
    <U as chrono::TimeZone>::Offset: Display,
{
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
        time_period: report_date_range.into(),
    }
}

#[cfg(test)]
mod test_cost_explorer_service {

    use super::*;
    use crate::reporting_date::ReportDateRange;
    use chrono::{Local, TimeZone};
    use cost_response_parser::{Cost, ReportedDateRange};
    use test_utils::{CostAndUsageClientStub, InputServiceCost};
    use tokio;

    #[tokio::test]
    async fn request_total_cost_correctly() {
        let client_stub = CostAndUsageClientStub {
            service_costs: None,
            total_cost: Some(String::from("1234.56")),
        };
        let report_date_range = ReportDateRange::new(Local.ymd(2021, 7, 23));
        let explorer = CostExplorerService::new(client_stub, report_date_range);

        let expected_total_cost = TotalCost {
            date_range: ReportedDateRange {
                start_date: Local.ymd(2021, 7, 1),
                end_date: Local.ymd(2021, 7, 23),
            },
            cost: Cost {
                amount: 1234.56,
                unit: String::from("USD"),
            },
        };

        let actual_total_cost = explorer.request_total_cost().await;

        assert_eq!(expected_total_cost, actual_total_cost);
    }

    #[tokio::test]
    async fn request_service_costs_correctly() {
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
            ServiceCost {
                service_name: String::from("Amazon Simple Storage Service"),
                cost: Cost {
                    amount: 1234.56,
                    unit: String::from("USD"),
                },
            },
            ServiceCost {
                service_name: String::from("Amazon Elastic Compute Cloud"),
                cost: Cost {
                    amount: 31415.92,
                    unit: String::from("USD"),
                },
            },
        ];

        let actual_service_costs = explorer.request_service_costs().await;

        assert_eq!(expected_service_costs, actual_service_costs);
    }
}

#[cfg(test)]
mod test_build_request {
    use super::*;
    use crate::reporting_date::ReportDateRange;
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
        let actual_request = build_cost_and_usage_request(&input_date_range, true);
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
        let actual_request = build_cost_and_usage_request(&input_date_range, false);

        assert_eq!(expected_request, actual_request);
    }
}
