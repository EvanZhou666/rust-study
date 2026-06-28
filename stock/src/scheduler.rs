use chrono::{Local, NaiveTime, Timelike};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::handlers::AppState;
use crate::scrapers::Scraper;
use crate::storage::Storage;

pub async fn run_scheduler(state: Arc<AppState>) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut need_initial = false;

    for config in state.configs.iter() {
        match state.storage.load_recent(&config.code, 1).await {
            Ok(prices) => {
                if prices.is_empty() || prices[0].date != today {
                    need_initial = true;
                    break;
                }
            }
            Err(_) => {
                need_initial = true;
                break;
            }
        }
    }

    if need_initial {
        tracing::info!("First collection of the day, starting now...");
        collect_all(&state).await;
    }

    loop {
        let until = duration_until_next_target(11, 0);
        tracing::info!("Next collection in {}", format_duration(until));
        sleep(until).await;
        tracing::info!("Scheduled collection started");
        collect_all(&state).await;
    }
}

pub async fn collect_all(state: &Arc<AppState>) {
    tracing::info!("Collecting all commodity data...");
    // 生猪，玉米，豆粕价格
    let zhujia_cfgs: Vec<_> = state.configs.iter().filter(|c| c.source == "zhujia").collect();
    let ppi_cfgs: Vec<_> = state.configs.iter().filter(|c| c.source == "ppi").collect();

    if let Some(first) = zhujia_cfgs.first() {
        match state.zhujia_scraper.fetch(first).await {
            Ok(all_prices) => {
                for config in &zhujia_cfgs {
                    let filtered: Vec<_> = all_prices.iter().filter(|p| p.name == config.name).cloned().collect();
                    if !filtered.is_empty() {
                        if let Err(e) = state.storage.save(&config.code, &filtered).await {
                            tracing::warn!("Save {} failed: {}", config.name, e);
                        } else {
                            tracing::info!("{} collected OK", config.name);
                        }
                    }
                }
            }
            Err(e) => tracing::error!("ZhuJia scrape failed: {}", e),
        }
    }

    for config in &ppi_cfgs {
        match state.ppi_scraper.fetch(config).await {
            Ok(prices) => {
                if let Err(e) = state.storage.save(&config.code, &prices).await {
                    tracing::warn!("Save {} failed: {}", config.name, e);
                } else {
                    tracing::info!("{} collected OK", config.name);
                }
            }
            Err(e) => tracing::error!("{} scrape failed: {}", config.name, e),
        }
    }

    tracing::info!("Collection complete");
}

fn duration_until_next_target(hour: u32, minute: u32) -> Duration {
    let now = Local::now();
    let target = NaiveTime::from_hms_opt(hour, minute, 0).unwrap_or(NaiveTime::from_hms_opt(11, 0, 0).unwrap());
    let now_time = NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    if now_time < target {
        Duration::from_secs((target - now_time).num_seconds().max(0) as u64)
    } else {
        let day_secs: i64 = 24 * 3600;
        let elapsed = now_time.num_seconds_from_midnight() as i64;
        let target_secs = target.num_seconds_from_midnight() as i64;
        Duration::from_secs((day_secs - elapsed + target_secs).max(0) as u64)
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
}
