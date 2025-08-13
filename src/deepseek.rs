use std::time::Duration;

use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::IntervalStream;
use tracing::instrument;

use crate::save_to_file;

#[derive(Debug, Serialize, Deserialize)]
struct UserBalance {
    pub is_available: bool,
    pub balance_infos: Vec<BalanceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BalanceInfo {
    pub currency: String,
    pub total_balance: String,
    pub granted_balance: String,
    pub topped_up_balance: String,
}

#[instrument(skip_all)]
async fn get_balance(key: &str, client: Client) -> anyhow::Result<UserBalance> {
    let url = "https://api.deepseek.com/user/balance";
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await?;
    if resp.status() != reqwest::StatusCode::OK {
        let text = resp.text().await?;
        return Err(anyhow::anyhow!(text));
    }
    let user_balance = resp.json::<UserBalance>().await?;

    Ok(user_balance)
}

#[instrument(skip_all)]
pub async fn check(keys: Vec<String>, query_per_sec: u32, client: Client) -> anyhow::Result<()> {
    let bar = ProgressBar::new(keys.len() as u64);
    let tasks = keys.into_iter().map(|key| {
        let client = client.clone();
        let bar = bar.clone();
        async move {
            let resp = get_balance(&key, client).await;
            bar.inc(1);
            (key, resp)
        }
    });
    let interval_tick = 1_f64 / (query_per_sec as f64);
    let interval = tokio::time::interval(Duration::from_secs_f64(interval_tick));
    let throttled_tasks = IntervalStream::new(interval).zip(futures::stream::iter(tasks));
    let resp = throttled_tasks
        .then(|(_, task)| task)
        .collect::<Vec<_>>()
        .await;
    bar.finish();
    check_resp(resp).await?;

    Ok(())
}

#[instrument(skip_all)]
async fn check_resp(resp: Vec<(String, anyhow::Result<UserBalance>)>) -> anyhow::Result<()> {
    let mut total = 0_f64;
    let mut have_banlance_keys = Vec::new();
    let mut no_balance_keys = Vec::new();
    let mut invalid_keys = Vec::new();
    let mut detail = Vec::new();
    detail.push("key, total_balance".to_string());
    for (key, resp) in resp.iter() {
        match resp {
            Ok(user) => {
                let total_balance = user.balance_infos[0]
                    .total_balance
                    .parse::<f64>()
                    .unwrap_or_default();
                if total_balance > 0_f64 {
                    have_banlance_keys.push(key);
                    total += total_balance;
                } else {
                    no_balance_keys.push(key);
                }
                detail.push(format!("{key}, {total_balance}"));
            }
            Err(err) => {
                tracing::error!("Error: {key}, {err}");
                invalid_keys.push(key);
                continue;
            }
        }
    }

    let prefix = "deepseek";
    save_to_file(have_banlance_keys, &format!("{prefix}_key")).await?;
    save_to_file(no_balance_keys, &format!("{prefix}_no_balance_keys")).await?;
    save_to_file(invalid_keys, &format!("{prefix}_invalid_keys")).await?;
    save_to_file(detail, &format!("{prefix}_detail.csv")).await?;
    tracing::info!("total: {total}");

    Ok(())
}
