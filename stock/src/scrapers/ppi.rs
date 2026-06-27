use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Local;
use scraper::{Html, Selector};

use super::Scraper;
use crate::models::commodity::{CommodityConfig, CommodityPrice};

pub struct PpiScraper {
    client: reqwest::Client,
}

impl PpiScraper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    fn parse_ppi_page(&self, html: &str, config: &CommodityConfig) -> Result<Vec<CommodityPrice>> {
        let document = Html::parse_document(html);
        let today = Local::now().format("%Y-%m-%d").to_string();

        // 尝试常见的历史价格表格选择器
        let selectors_to_try = [
            "table.history-table tbody tr",
            "table.price-table tbody tr",
            "table tbody tr:first-child",
            "div.price-box .price",
            "span.price",
        ];

        for sel_str in &selectors_to_try {
            let selector = match Selector::parse(sel_str) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                // 尝试提取数字
                let cleaned: String = text
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect();

                if let Ok(price) = cleaned.parse::<f64>() {
                    return Ok(vec![CommodityPrice {
                        date: today,
                        name: config.name.clone(),
                        price,
                        unit: config.unit.clone(),
                        change: None,
                        change_percent: None,
                    }]);
                }
            }
        }

        Err(anyhow!(
            "未能在 100ppi.com 找到 {} 的价格数据，请检查 CSS 选择器。URL: {}",
            config.name,
            config.url
        ))
    }
}

#[async_trait]
impl Scraper for PpiScraper {
    async fn fetch(&self, config: &CommodityConfig) -> Result<Vec<CommodityPrice>> {
        let resp = self.client.get(&config.url).send().await?;
        let html = resp.text().await?;
        self.parse_ppi_page(&html, config)
    }

    fn source(&self) -> &str {
        "ppi"
    }
}
