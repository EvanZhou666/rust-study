use axum::extract::State;
use axum::response::{Html, IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use tera::Context;

use super::AppState;
use crate::analysis::trend::analyze_trend;
use crate::models::commodity::TrendDirection;
use crate::storage::Storage;

#[derive(Serialize)]
struct CommodityRow {
    code: String,
    name: String,
    price: String,
    unit: String,
    change_str: String,
    change_class: String,
    trend_label: String,
    trend_class: String,
}

#[derive(Serialize)]
struct DashboardContext {
    title: String,
    commodities: Vec<CommodityRow>,
    last_collected: String,
}

pub async fn dashboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut rows = Vec::new();
    let last = String::new();

    for config in state.configs.iter() {
        let prices = state.storage.load_recent(&config.code, 30).await.unwrap_or_default();
        let trend = analyze_trend(&config.code, &prices);

        let (price_str, change_str, change_class) = if let Some(latest) = prices.last() {
            let p = format!("{:.2}", latest.price);
            let (cs, cc) = match latest.change_percent {
                Some(cp) if cp > 0.0 => (format!("+{:.2}%", cp), "text-success"),
                Some(cp) if cp < 0.0 => (format!("{:.2}%", cp), "text-danger"),
                _ => ("--".into(), "text-muted"),
            };
            (p, cs, cc.to_string())
        } else {
            ("--".into(), "--".into(), "text-muted".into())
        };

        let (trend_label, trend_class) = match &trend {
            Some(t) => match t.direction {
                TrendDirection::Up => (format!("up {} {}", t.direction, t.consecutive_days), "badge bg-danger"),
                TrendDirection::Down => (format!("down {} {}", t.direction, t.consecutive_days), "badge bg-success"),
                TrendDirection::Flat => ("flat".into(), "badge bg-secondary"),
            },
            None => ("nodata".into(), "badge bg-secondary".into()),
        };

        rows.push(CommodityRow {
            code: config.code.clone(),
            name: config.name.clone(),
            price: price_str,
            unit: config.unit.clone(),
            change_str,
            change_class,
            trend_label,
            trend_class: trend_class.to_string(),
        });
    }

    let ctx = DashboardContext {
        title: "Commodity Dashboard".into(),
        commodities: rows,
        last_collected: if last.is_empty() { "N/A".into() } else { last },
    };

    let rendered = state.tera.render("dashboard.html", &Context::from_serialize(&ctx).unwrap());
    match rendered {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            Html(format!("<h1>Template Error</h1><pre>{}</pre>", e)).into_response()
        }
    }
}
