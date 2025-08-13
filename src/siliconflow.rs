use std::time::Duration;

use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::IntervalStream;
use tracing::instrument;

use crate::save_to_file;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub balance: String,
    pub status: String,
    pub charge_balance: String,
    pub total_balance: String,
}

#[instrument(skip_all)]
async fn get_userinfo(key: &str, client: Client) -> anyhow::Result<UserInfo> {
    let url = "https://api.siliconflow.cn/v1/user/info";
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await?;
    let status_code = resp.status();
    if status_code != reqwest::StatusCode::OK {
        let err_text = resp.text().await?;
        return Err(anyhow::anyhow!(err_text));
    }
    let user_info = resp.json::<UserInfo>().await?;
    Ok(user_info)
}

#[instrument(skip_all)]
pub async fn check(keys: Vec<String>, query_per_sec: u32, client: Client) -> anyhow::Result<()> {
    let bar = ProgressBar::new(keys.len() as u64);
    let tasks = keys.into_iter().map(|key| {
        let client = client.clone();
        let bar = bar.clone();
        async move {
            let resp = get_userinfo(&key, client).await;
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
async fn check_resp(resp: Vec<(String, anyhow::Result<UserInfo>)>) -> anyhow::Result<()> {
    let mut total = 0_f64;
    let mut total_pro = 0_f64;
    let mut pro_keys = Vec::new();
    let mut have_banlance_keys = Vec::new();
    let mut no_balance_keys = Vec::new();
    let mut disable_keys = Vec::new();
    let mut invalid_keys = Vec::new();
    let mut detail = Vec::new();
    detail.push("key, charge_balance, total_balance".to_string());
    for (key, resp) in resp.iter() {
        match resp {
            Ok(user) => {
                if user.data.status == "disable" {
                    disable_keys.push(key);
                    continue;
                }

                let charge_balance = user.data.charge_balance.parse::<f64>().unwrap_or_default();
                let total_balance = user.data.total_balance.parse::<f64>().unwrap_or_default();
                if charge_balance > 0_f64 {
                    pro_keys.push(key);
                    total_pro += charge_balance;
                }
                if total_balance > 0_f64 {
                    have_banlance_keys.push(key);
                    total += total_balance;
                } else {
                    no_balance_keys.push(key);
                }
                detail.push(format!("{key}, {charge_balance}, {total_balance}"));
            }
            Err(err) => {
                tracing::error!("Error: {key}, {err}");
                invalid_keys.push(key);
                continue;
            }
        }
    }
    let prefix = "siliconflow";
    save_to_file(pro_keys, &format!("{prefix}_pro_key")).await?;
    save_to_file(have_banlance_keys, &format!("{prefix}_key")).await?;
    save_to_file(no_balance_keys, &format!("{prefix}_no_balance_keys")).await?;
    save_to_file(disable_keys, &format!("{prefix}_disable_keys")).await?;
    save_to_file(invalid_keys, &format!("{prefix}_invalid_keys")).await?;
    save_to_file(detail, &format!("{prefix}_detail.csv")).await?;
    tracing::info!("total: {total}, pro: {total_pro}");
    Ok(())
}
