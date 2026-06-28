mod config;
mod models;
mod scrapers;
mod storage;
mod analysis;
mod handlers;
mod scheduler;

use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::services::ServeDir;

use handlers::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new("data/commodities"));
    // 增加引用计数，并非数据拷贝
    let scheduler_state = state.clone();

    tokio::spawn(async move {
        scheduler::run_scheduler(scheduler_state).await;
    });

    let router = Router::new()
        .route("/", get(handlers::dashboard::dashboard))
        .route("/commodity/{code}", get(handlers::commodity_detail::commodity_detail))
        .route("/api/refresh", get(handlers::api::refresh_all))
        .route("/api/commodity/{code}/prices", get(handlers::api::commodity_prices))
        .route("/api/commodity/{code}/chart", get(handlers::api::commodity_chart))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:30001").await.unwrap();
    tracing::info!("Server started: http://localhost:30001");
    axum::serve(listener, router).await.unwrap();
}
