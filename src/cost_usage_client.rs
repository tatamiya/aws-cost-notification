use rusoto_ce::{
    CostExplorer, CostExplorerClient, GetCostAndUsageError, GetCostAndUsageRequest,
    GetCostAndUsageResponse,
};
use rusoto_core::{Region, RusotoError};

use async_trait::async_trait;

#[async_trait]
pub trait GetCostAndUsage {
    async fn get_cost_and_usage(
        &self,
        input: GetCostAndUsageRequest,
    ) -> Result<GetCostAndUsageResponse, RusotoError<GetCostAndUsageError>>;
}

pub struct CostAndUsageClient(CostExplorerClient);

impl CostAndUsageClient {
    pub fn new(region: Region) -> Self {
        CostAndUsageClient(CostExplorerClient::new(region))
    }
}

#[async_trait]
impl GetCostAndUsage for CostAndUsageClient {
    async fn get_cost_and_usage(
        &self,
        input: GetCostAndUsageRequest,
    ) -> Result<GetCostAndUsageResponse, RusotoError<GetCostAndUsageError>> {
        (&self.0).get_cost_and_usage(input).await
    }
}
