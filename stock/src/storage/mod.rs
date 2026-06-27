use anyhow::Result;
use async_trait::async_trait;
use crate::models::commodity::CommodityPrice;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn save(&self, commodity_code: &str, prices: &[CommodityPrice]) -> Result<()>;
    async fn load(&self, commodity_code: &str) -> Result<Vec<CommodityPrice>>;
    async fn load_recent(&self, commodity_code: &str, days: u32) -> Result<Vec<CommodityPrice>>;
}

pub mod json_store;
