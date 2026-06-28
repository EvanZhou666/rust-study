use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Local;
use regex::Regex;
use scraper::{Html, Selector};

use super::Scraper;
use crate::models::commodity::{CommodityConfig, CommodityPrice};

pub struct PpiScraper {
    client: reqwest::Client,
}

impl PpiScraper {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Mobile Safari/537.36".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::REFERER,
            "https://m1.100ppi.com/".parse().unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    fn parse_ppi_page(&self, html: &str, config: &CommodityConfig) -> Result<Vec<CommodityPrice>> {
        let document = Html::parse_document(html);
        let today = Local::now().format("%Y-%m-%d").to_string();

        // 策略1: 从 <meta name="description"> 提取价格（最可靠）
        let price_re = Regex::new(r"基准价为([\d.]+)元/吨").unwrap();
        if let Ok(meta_sel) = Selector::parse("meta[name=\"description\"]") {
            if let Some(meta) = document.select(&meta_sel).next() {
                if let Some(content) = meta.value().attr("content") {
                    if let Some(caps) = price_re.captures(content) {
                        if let Ok(price) = caps[1].parse::<f64>() {
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
            }
        }

        // 策略2: 从 <p class="wsdfont1"> 提取价格（备用）
        if let Ok(price_sel) = Selector::parse("p.wsdfont1") {
            if let Some(el) = document.select(&price_sel).next() {
                let text = el.text().collect::<String>().trim().to_string();
                let cleaned: String = text.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
                if let Ok(price) = cleaned.parse::<f64>() {
                    let change_percent = if let Ok(chg_sel) = Selector::parse("p.price-fb04") {
                        document.select(&chg_sel).next().and_then(|el| {
                            let t = el.text().collect::<String>().trim().replace('%', "");
                            t.parse::<f64>().ok()
                        })
                    } else {
                        None
                    };

                    return Ok(vec![CommodityPrice {
                        date: today,
                        name: config.name.clone(),
                        price,
                        unit: config.unit.clone(),
                        change: None,
                        change_percent,
                    }]);
                }
            }
        }

        // 策略3: 从 <div class="sP_middle"> 提取价格（最后备用）
        if let Ok(mid_sel) = Selector::parse("div.sP_middle") {
            if let Some(el) = document.select(&mid_sel).next() {
                let text = el.text().collect::<String>().trim().to_string();
                let cleaned: String = text
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.')
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
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("HTTP {} for {}", status, config.url));
        }
        let html = resp.text().await?;
        self.parse_ppi_page(&html, config)
    }

    fn source(&self) -> &str {
        "ppi"
    }
}
