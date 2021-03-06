/// Parse the CostExplorer API Response
pub mod cost_response_parser;
/// Client to retrieve the AWS costs.
/// It wraps [CostExplorerClient](https://docs.rs/rusoto_ce/0.47.0/rusoto_ce/struct.CostExplorerClient.html).
pub mod cost_usage_client;
/// Functions and structs used for tests.
pub mod test_utils;

use chrono::TimeZone;
use rusoto_ce::{GetCostAndUsageRequest, GroupDefinition};
use std::fmt::Display;

use crate::reporting_date::ReportDateRange;
use cost_response_parser::{ServiceCost, TotalCost};
use cost_usage_client::GetCostAndUsage;

/// Object to send request to CostExplorer API and retrieve AWS costs.
pub struct CostExplorerService<C: GetCostAndUsage, T>
where
    T: TimeZone,
    <T as chrono::TimeZone>::Offset: Display,
{
    /// CostAndUsageClient
    client: C,
    /// The date period to retrieve the costs.
    report_date_range: ReportDateRange<T>,
}
impl<C: GetCostAndUsage, T> CostExplorerService<C, T>
where
    T: TimeZone,
    <T as chrono::TimeZone>::Offset: Display,
{
    /// Constructor method
    pub fn new(client: C, report_date_range: ReportDateRange<T>) -> Self {
        CostExplorerService {
            client: client,
            report_date_range: report_date_range,
        }
    }

    /// Sends request to GetCostAndUsage endpoint of CostExplorer API
    /// and returns parsed total cost.
    pub async fn request_total_cost(&self) -> TotalCost {
        let request: GetCostAndUsageRequest =
            build_cost_and_usage_request(&self.report_date_range, true);

        let res = self.client.get_cost_and_usage(request).await.unwrap();
        res.into()
    }

    /// Sends request to GetCostAndUsage endpoint of CostExplorer API
    /// and returns a vector of parsed service costs.
    pub async fn request_service_costs(&self) -> Vec<ServiceCost> {
        let request: GetCostAndUsageRequest =
            build_cost_and_usage_request(&self.report_date_range, false);
        let res = self.client.get_cost_and_usage(request).await.unwrap();
        ServiceCost::from_response(&res)
    }
}

/// Build the request object of the CostExplorer API.
/// The data aquisition period is designated by `report_date_range`.
/// If `is_total` is true, it builds request for total cost.
/// Otherwise, it requests the costs grouped by AWS services.
fn build_cost_and_usage_request<T>(
    report_date_range: &ReportDateRange<T>,
    is_total: bool,
) -> GetCostAndUsageRequest
where
    T: TimeZone,
    <T as chrono::TimeZone>::Offset: Display,
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
