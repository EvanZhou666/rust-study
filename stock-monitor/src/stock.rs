use anyhow::{bail, Context};
use encoding_rs::GBK;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub open: f64,
    pub prev_close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub amount: f64,
    pub date: String,
    pub time: String,
}

impl StockQuote {
    pub fn change(&self) -> f64 {
        self.price - self.prev_close
    }

    pub fn change_percent(&self) -> f64 {
        if self.prev_close.abs() < f64::EPSILON {
            0.0
        } else {
            self.change() / self.prev_close * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

pub struct SinaProvider {
    client: reqwest::blocking::Client,
}

impl SinaProvider {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 stock-monitor/0.1")
            .build()
            .context("build http client")?;
        Ok(Self { client })
    }

    pub fn fetch_quotes(&self, symbols: &[String]) -> anyhow::Result<Vec<StockQuote>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let list = symbols.join(",");
        let url = format!("https://hq.sinajs.cn/list={list}");
        let bytes = self
            .client
            .get(url)
            .header("Referer", "https://finance.sina.com.cn/")
            .send()
            .context("request sina quotes")?
            .bytes()
            .context("read sina quote body")?;

        let (text, _, _) = GBK.decode(&bytes);
        parse_quotes(&text)
    }

    pub fn fetch_intraday(&self, symbol: &str) -> anyhow::Result<Vec<Candle>> {
        let url = format!(
            "https://quotes.sina.cn/cn/api/jsonp_v2.php/=/CN_MarketDataService.getKLineData?symbol={symbol}&scale=5&ma=no&datalen=48"
        );
        let text = self
            .client
            .get(url)
            .header("Referer", "https://finance.sina.com.cn/")
            .send()
            .context("request sina intraday candles")?
            .text()
            .context("read sina candle body")?;

        parse_candles(&text)
    }
}

fn parse_quotes(text: &str) -> anyhow::Result<Vec<StockQuote>> {
    let mut quotes = Vec::new();

    for line in text.lines() {
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        let Some(symbol) = left.strip_prefix("var hq_str_") else {
            continue;
        };
        let data = right.trim().trim_end_matches(';').trim_matches('"');
        if data.is_empty() {
            continue;
        }

        let fields: Vec<&str> = data.split(',').collect();
        if fields.len() < 32 {
            continue;
        }

        quotes.push(StockQuote {
            symbol: symbol.to_owned(),
            name: fields[0].to_owned(),
            open: parse_f64(fields[1]),
            prev_close: parse_f64(fields[2]),
            price: parse_f64(fields[3]),
            high: parse_f64(fields[4]),
            low: parse_f64(fields[5]),
            volume: parse_u64(fields[8]),
            amount: parse_f64(fields[9]),
            date: fields[30].to_owned(),
            time: fields[31].to_owned(),
        });
    }

    Ok(quotes)
}

fn parse_candles(text: &str) -> anyhow::Result<Vec<Candle>> {
    let start = text.find('[').context("find candle json start")?;
    let end = text.rfind(']').context("find candle json end")?;
    if end < start {
        bail!("invalid candle json range");
    }

    let json = &text[start..=end];
    let rows: Vec<SinaCandle> = serde_json::from_str(json).context("parse candle json")?;
    Ok(rows
        .into_iter()
        .map(|row| Candle {
            time: row.day,
            open: parse_f64(&row.open),
            high: parse_f64(&row.high),
            low: parse_f64(&row.low),
            close: parse_f64(&row.close),
            volume: parse_u64(&row.volume),
        })
        .collect())
}

fn parse_f64(value: &str) -> f64 {
    value.trim().parse().unwrap_or(0.0)
}

fn parse_u64(value: &str) -> u64 {
    value.trim().parse().unwrap_or(0)
}

#[derive(Debug, Deserialize)]
struct SinaCandle {
    day: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}
