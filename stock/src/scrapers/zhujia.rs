use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Local;
use scraper::{Html, Selector};

use super::Scraper;
use crate::models::commodity::{CommodityConfig, CommodityPrice};

pub struct ZhuJiaScraper {
    client: reqwest::Client,
}

impl ZhuJiaScraper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub fn parse_zhujia_all(&self, html: &str) -> Result<Vec<CommodityPrice>> {
        let document = Html::parse_document(html);
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut results = Vec::new();

        let tr_sel = Selector::parse("table tbody tr")
            .map_err(|e| anyhow!("CSS error: {}", e))?;
        let td_sel = Selector::parse("td")
            .map_err(|e| anyhow!("CSS error: {}", e))?;

        for row in document.select(&tr_sel) {
            let cells: Vec<_> = row.select(&td_sel).collect();
            if cells.len() < 3 {
                continue;
            }
            let name_text = cells[0].text().collect::<String>();
            let price_text = cells[1].text().collect::<String>().trim().to_string();

            let (commodity_name, commodity_unit) = if name_text.contains("生猪") {
                ("生猪", "元/公斤")
            } else if name_text.contains("玉米") {
                ("玉米", "元/吨")
            } else if name_text.contains("豆粕") {
                ("豆粕", "元/吨")
            } else {
                continue;
            };

            if let Ok(price) = price_text.parse::<f64>() {
                results.push(CommodityPrice {
                    date: today.clone(),
                    name: commodity_name.to_string(),
                    price,
                    unit: commodity_unit.to_string(),
                    change: None,
                    change_percent: None,
                });
            }
        }

        if results.is_empty() {
            Err(anyhow!("未能在猪价网找到任何商品价格数据，请检查 CSS 选择器"))
        } else {
            Ok(results)
        }
    }
}

#[async_trait]
impl Scraper for ZhuJiaScraper {
    async fn fetch(&self, config: &CommodityConfig) -> Result<Vec<CommodityPrice>> {
        let resp = self.client.get(&config.url).send().await?;
        let html = resp.text().await?;
        let all = self.parse_zhujia_all(&html)?;
        let filtered: Vec<_> = all.into_iter().filter(|p| p.name == config.name).collect();
        if filtered.is_empty() {
            Err(anyhow!("未找到 {} 的价格数据", config.name))
        } else {
            Ok(filtered)
        }
    }

    fn source(&self) -> &str {
        "zhujia"
    }
}
