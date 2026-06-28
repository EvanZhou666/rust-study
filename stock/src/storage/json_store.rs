use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Local;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;

use super::Storage;
use crate::models::commodity::{CommodityMeta, CommodityPrice, CommodityStore};

pub struct JsonStore {
    data_dir: PathBuf,
}

impl JsonStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn file_path(&self, commodity_code: &str) -> PathBuf {
        self.data_dir.join(commodity_code).join("prices.json")
    }

    async fn load_store(&self, commodity_code: &str) -> Result<CommodityStore> {
        let path = self.file_path(commodity_code);
        if !path.exists() {
            return Ok(CommodityStore {
                commodity: CommodityMeta {
                    name: commodity_code.into(),
                    code: commodity_code.into(),
                    unit: String::new(),
                },
                prices: Vec::new(),
                last_updated: String::new(),
            });
        }
        let content = fs::read_to_string(&path).await
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let store: CommodityStore = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(store)
    }

    async fn save_store(&self, store: &CommodityStore) -> Result<()> {
        let code = &store.commodity.code;
        let dir = self.data_dir.join(code);
        fs::create_dir_all(&dir).await?;
        let path = self.file_path(code);
        let content = serde_json::to_string_pretty(store)?;
        fs::write(&path, content).await
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }
}

#[async_trait]
impl Storage for JsonStore {
    async fn save(&self, commodity_code: &str, prices: &[CommodityPrice]) -> Result<()> {
        let mut store = self.load_store(commodity_code).await?;
        let existing_dates: HashSet<String> = store.prices.iter().map(|p| p.date.clone()).collect();
        for price in prices {
            if !existing_dates.contains(&price.date) {
                store.prices.push(price.clone());
            }
        }
        store.prices.sort_by(|a, b| a.date.cmp(&b.date));

        // 用最新一条价格数据刷新元数据
        if let Some(latest) = store.prices.last() {
            store.commodity.name = latest.name.clone();
            store.commodity.unit = latest.unit.clone();
        }

        store.last_updated = Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string();
        self.save_store(&store).await
    }

    async fn load(&self, commodity_code: &str) -> Result<Vec<CommodityPrice>> {
        let store = self.load_store(commodity_code).await?;
        Ok(store.prices)
    }

    async fn load_recent(&self, commodity_code: &str, days: u32) -> Result<Vec<CommodityPrice>> {
        let store = self.load_store(commodity_code).await?;
        let total = store.prices.len();
        let skip = total.saturating_sub(days as usize);
        Ok(store.prices.into_iter().skip(skip).collect())
    }
}