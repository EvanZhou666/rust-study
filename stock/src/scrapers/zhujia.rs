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

    // 解析全国汇总价格卡片：ul.zhujia-hd > li[title="..."]
    // 每张卡片：<span>名称</span> <b>价格</b> <i><span>涨跌额元</span>/单位</i>
    pub fn parse_zhujia_cards(&self, html: &str) -> Result<Vec<CommodityPrice>> {
        let document = Html::parse_document(html);
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut results = Vec::new();

        // 选中价格卡片列表中的每个 <li>，每张卡片代表一个商品（外三元/玉米/豆粕）
        let li_sel = Selector::parse("ul.zhujia-hd li").map_err(|e| anyhow!("CSS error: {}", e))?;
        // 选中卡片内的 <b> 标签，里面是价格数值，如 <b class="red">9.52</b>
        let b_sel = Selector::parse("b").map_err(|e| anyhow!("CSS error: {}", e))?;
        // 选中 <i> 下的 <span>，里面是涨跌额文字，如 <span class="red">0.05元</span> 或 <span>报价平稳</span>
        let i_span_sel = Selector::parse("i > span").map_err(|e| anyhow!("CSS error: {}", e))?;

        for li in document.select(&li_sel) {
            let title = match li.value().attr("title") {
                Some(t) => t.to_string(),
                None => continue,
            };

            let (commodity_name, commodity_unit) = match title.as_str() {
                "外三元" => ("生猪", "元/公斤"),
                "玉米" => ("玉米", "元/吨"),
                "豆粕" => ("豆粕", "元/吨"),
                _ => continue,
            };

            let price_text = match li.select(&b_sel).next() {
                Some(el) => el.text().collect::<String>().trim().to_string(),
                None => continue,
            };
            let price: f64 = match price_text.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let change: Option<f64> = li
                .select(&i_span_sel)
                .next()
                .and_then(|el| {
                    let t = el.text().collect::<String>();
                    let cleaned: String = t
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                        .collect();
                    cleaned.parse::<f64>().ok()
                });

            results.push(CommodityPrice {
                date: today.clone(),
                name: commodity_name.to_string(),
                price,
                unit: commodity_unit.to_string(),
                change,
                change_percent: None,
            });
        }

        if results.is_empty() {
            Err(anyhow!("未能在猪价网找到任何商品价格数据"))
        } else {
            tracing::info!("获取生猪/玉米/豆粕价格{:#?}", results);
            Ok(results)
        }
    }
}

#[async_trait]
impl Scraper for ZhuJiaScraper {
    async fn fetch(&self, config: &CommodityConfig) -> Result<Vec<CommodityPrice>> {
        let resp = self.client.get(&config.url).send().await?;
        let html = resp.text().await?;
        // 一次抓取返回页面上所有商品价格，由 scheduler 按 name 分发
        self.parse_zhujia_cards(&html)
    }

    fn source(&self) -> &str {
        "zhujia"
    }
}