use async_trait::async_trait;
use anyhow::Result;
use crate::models::commodity::{CommodityConfig, CommodityPrice};

#[async_trait]
#[allow(dead_code)]
pub trait Scraper: Send + Sync {
    async fn fetch(&self, config: &CommodityConfig) -> Result<Vec<CommodityPrice>>;
    fn source(&self) -> &str;
}

pub mod zhujia;
pub mod ppi;
