use rusoto_ce::{
    CostExplorer, CostExplorerClient, GetCostAndUsageError, GetCostAndUsageRequest,
    GetCostAndUsageResponse,
};
use rusoto_core::{Region, RusotoError};

use async_trait::async_trait;

/// Trait which picks up [get_cost_and_usage](https://docs.rs/rusoto_ce/0.47.0/rusoto_ce/trait.CostExplorer.html#tymethod.get_cost_and_usage) method from [rusoto_ce::CostExplorer](https://docs.rs/rusoto_ce/0.47.0/rusoto_ce/trait.CostExplorer.html) trait.
#[async_trait]
pub trait GetCostAndUsage {
    /// Retrieves AWS cost ans usage. [See this](https://docs.rs/rusoto_ce/0.47.0/rusoto_ce/struct.CostExplorerClient.html#method.get_anomaly_subscriptions)
    async fn get_cost_and_usage(
        &self,
        input: GetCostAndUsageRequest,
    ) -> Result<GetCostAndUsageResponse, RusotoError<GetCostAndUsageError>>;
}

/// Wrapper of [rusoto_ce::CostExplorerClient](https://docs.rs/rusoto_ce/0.47.0/rusoto_ce/struct.CostExplorerClient.html).
/// It implements only [get_cost_and_usage](https://docs.rs/rusoto_ce/0.47.0/rusoto_ce/struct.CostExplorerClient.html#method.get_anomaly_subscriptions) method
/// to send a request to [GetCostAndUsage endpoint](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_GetCostAndUsage.html)
/// of CostExplorer API.
pub struct CostAndUsageClient(CostExplorerClient);

impl CostAndUsageClient {
    pub fn new() -> Self {
        // NOTE: Region must not be ap-northeast-1
        // because endpoint https://ce.ap-northeast1.amazonaws.com/ does not exist
        CostAndUsageClient(CostExplorerClient::new(Region::UsEast1))
    }
}

#[async_trait]
impl GetCostAndUsage for CostAndUsageClient {
    /// Send a request to [GetCostAndUsage endpoint](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_GetCostAndUsage.html)
    /// of CostExplorer API.
    async fn get_cost_and_usage(
        &self,
        input: GetCostAndUsageRequest,
    ) -> Result<GetCostAndUsageResponse, RusotoError<GetCostAndUsageError>> {
        (&self.0).get_cost_and_usage(input).await
    }
}
