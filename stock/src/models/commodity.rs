use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityPrice {
    pub date: String,
    pub name: String,
    pub price: f64,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityConfig {
    pub name: String,
    pub code: String,
    pub source: String,
    pub url: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Up,
    Down,
    Flat,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::Up => write!(f, "上涨"),
            TrendDirection::Down => write!(f, "下跌"),
            TrendDirection::Flat => write!(f, "震荡"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendResult {
    pub commodity_code: String,
    pub period_days: u32,
    pub direction: TrendDirection,
    pub consecutive_days: u32,
    pub change_percent: f64,
    pub short_ma: f64,
    pub medium_ma: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityMeta {
    pub name: String,
    pub code: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityStore {
    pub commodity: CommodityMeta,
    pub prices: Vec<CommodityPrice>,
    pub last_updated: String,
}
