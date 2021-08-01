use async_trait::async_trait;
use rusoto_ce::*;
use rusoto_core::RusotoError;
use std::collections::HashMap;

use crate::cost_explorer::cost_usage_client::GetCostAndUsage;

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
}
impl From<InputServiceCost> for Group {
    fn from(from: InputServiceCost) -> Group {
        let mut metrics = HashMap::new();
        metrics.insert(
            String::from("AmortizedCost"),
            MetricValue {
                amount: Some(from.cost.clone()),
                unit: Some(String::from("USD")),
            },
        );
        Group {
            keys: Some(vec![from.service_name.clone()]),
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
        Some(service_costs) => Some(service_costs.iter().map(|x| x.clone().into()).collect()),
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
