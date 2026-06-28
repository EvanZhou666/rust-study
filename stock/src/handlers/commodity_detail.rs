use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use tera::Context;

use super::AppState;
use crate::analysis::trend::analyze_trend;
use crate::models::commodity::CommodityPrice;
use crate::storage::Storage;

#[derive(Serialize)]
struct DetailContext {
    title: String,
    commodity_name: String,
    commodity_code: String,
    unit: String,
    prices: Vec<CommodityPrice>,
    trend_consecutive: String,
    trend_direction: String,
    trend_change: String,
    trend_short_ma: String,
    trend_medium_ma: String,
    trend_period: u32,
}

pub async fn commodity_detail(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let config = state
        .configs
        .iter()
        .find(|c| c.code == code)
        .cloned();

    let config = match config {
        Some(c) => c,
        None => return Html("<h1>Not Found</h1>".to_string()).into_response(),
    };

    let prices = state.storage.load(&code).await.unwrap_or_default();
    let trend = analyze_trend(&code, &prices);

    let ctx = DetailContext {
        title: format!("{} - Detail", config.name),
        commodity_name: config.name.clone(),
        commodity_code: code,
        unit: config.unit.clone(),
        prices,
        trend_consecutive: trend.as_ref().map_or("--".into(), |t| format!("{} days", t.consecutive_days)),
        trend_direction: trend.as_ref().map_or("N/A".into(), |t| t.direction.to_string()),
        trend_change: trend.as_ref().map_or("--".into(), |t| format!("{:.2}%", t.change_percent)),
        trend_short_ma: trend.as_ref().map_or("--".into(), |t| format!("{:.2}", t.short_ma)),
        trend_medium_ma: trend.as_ref().map_or("--".into(), |t| format!("{:.2}", t.medium_ma)),
        trend_period: trend.as_ref().map_or(0, |t| t.period_days),
    };

    let rendered = state.tera.render("commodity_detail.html", &Context::from_serialize(&ctx).unwrap());
    match rendered {
        Ok(html) => Html(html).into_response(),
        Err(e) => { 
            tracing::error!("Render error: {:#}", e);
            Html(format!("<h1>Render Error</h1><pre>{:#}</pre>", e)).into_response() 
        },
    }
}

