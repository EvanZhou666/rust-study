use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use tera::Context;

use super::AppState;
use crate::analysis::trend::analyze_trend;
use crate::scrapers::Scraper;
use crate::storage::Storage;

#[derive(Serialize)]
struct ChartData {
    labels: Vec<String>,
    prices: Vec<f64>,
}

pub async fn refresh_all(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut messages = Vec::new();

    let zhujia_configs: Vec<_> = state.configs.iter().filter(|c| c.source == "zhujia").collect();
    let ppi_configs: Vec<_> = state.configs.iter().filter(|c| c.source == "ppi").collect();

    if let Some(first) = zhujia_configs.first() {
        match state.zhujia_scraper.fetch(first).await {
            Ok(prices) => {
                for config in &zhujia_configs {
                    let filtered: Vec<_> = prices
                        .iter()
                        .filter(|p| p.name == config.name)
                        .cloned()
                        .collect();
                    if !filtered.is_empty() {
                        if let Err(e) = state.storage.save(&config.code, &filtered).await {
                            messages.push(format!("Save {} failed: {}", config.name, e));
                        } else {
                            messages.push(format!("{} collected OK", config.name));
                        }
                    }
                }
            }
            Err(e) => messages.push(format!("ZhuJia scrape failed: {}", e)),
        }
    }

    for config in &ppi_configs {
        match state.ppi_scraper.fetch(config).await {
            Ok(prices) => {
                if let Err(e) = state.storage.save(&config.code, &prices).await {
                    messages.push(format!("Save {} failed: {}", config.name, e));
                } else {
                    messages.push(format!("{} collected OK", config.name));
                }
            }
            Err(e) => messages.push(format!("{} scrape failed: {}", config.name, e)),
        }
    }

    let result = messages.join("<br>");
    Html(format!("<div id=\"refresh-status\" class=\"alert alert-info\">{}</div>", result))
}

pub async fn commodity_prices(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let config = match state.configs.iter().find(|c| c.code == code) {
        Some(c) => c,
        None => return Html("Not Found".to_string()),
    };

    let prices = state.storage.load(&code).await.unwrap_or_default();
    let trend = analyze_trend(&code, &prices);

    let mut ctx = Context::new();
    ctx.insert("prices", &prices);
    ctx.insert("commodity_name", &config.name);
    ctx.insert("commodity_code", &config.code);
    ctx.insert("unit", &config.unit);
    if let Some(t) = &trend {
        ctx.insert("trend_direction", &t.direction.to_string());
    }

    match state.tera.render("partials/price_table.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<p>Render Error: {}</p>", e)),
    }
}

pub async fn commodity_chart(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let prices = state.storage.load(&code).await.unwrap_or_default();

    let labels: Vec<String> = prices.iter().map(|p| p.date.clone()).collect();
    let data: Vec<f64> = prices.iter().map(|p| p.price).collect();

    Json(ChartData {
        labels,
        prices: data,
    })
}
