use crate::models::commodity::{CommodityPrice, TrendDirection, TrendResult};

pub fn analyze_trend(
    commodity_code: &str,
    prices: &[CommodityPrice],
) -> Option<TrendResult> {
    if prices.len() < 20 {
        return None;
    }

    let recent: Vec<f64> = prices.iter().map(|p| p.price).collect();

    let short_ma = simple_moving_average(&recent, 5);
    let medium_ma = simple_moving_average(&recent, 20);

    let mut consecutive_days: u32 = 0;
    let mut direction = TrendDirection::Flat;
    let len = recent.len();
    if len >= 2 {
        let latest_diff = recent[len - 1] - recent[len - 2];
        if latest_diff > 0.0 {
            direction = TrendDirection::Up;
        } else if latest_diff < 0.0 {
            direction = TrendDirection::Down;
        }

        for i in (1..len).rev() {
            let diff = recent[i] - recent[i - 1];
            match direction {
                TrendDirection::Up if diff > 0.0 => consecutive_days += 1,
                TrendDirection::Down if diff < 0.0 => consecutive_days += 1,
                _ => break,
            }
        }
    }

    let first = recent[0];
    let last = recent[recent.len() - 1];
    let change_percent = if first != 0.0 {
        ((last - first) / first) * 100.0
    } else {
        0.0
    };

    let final_direction = if short_ma > medium_ma && consecutive_days >= 3 && change_percent > 2.0 {
        TrendDirection::Up
    } else if short_ma < medium_ma && consecutive_days >= 3 && change_percent < -2.0 {
        TrendDirection::Down
    } else {
        TrendDirection::Flat
    };

    Some(TrendResult {
        commodity_code: commodity_code.to_string(),
        period_days: len as u32,
        direction: final_direction,
        consecutive_days,
        change_percent,
        short_ma,
        medium_ma,
    })
}

fn simple_moving_average(prices: &[f64], window: usize) -> f64 {
    if prices.len() < window {
        return prices.iter().sum::<f64>() / prices.len() as f64;
    }
    let window_slice = &prices[prices.len() - window..];
    window_slice.iter().sum::<f64>() / window as f64
}
