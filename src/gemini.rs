use std::time::Duration;

use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use serde_json::json;
use tokio_stream::wrappers::IntervalStream;
use tracing::instrument;

use crate::save_to_file;

async fn say_hi(key: &str, model: &str, client: Client) -> anyhow::Result<u16> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={key}"
    );
    let resp = client
        .post(url)
        .json(&json!({
            "contents": [{
                "parts": [{"text": "hi"}]
            }]
        }))
        .send()
        .await?;
    tracing::debug!("{key}, {model}, {}", resp.status());
    Ok(resp.status().as_u16())
}

async fn list_model(key: &str, client: Client) -> anyhow::Result<u16> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={key}");
    let resp = client.get(url).send().await?;
    Ok(resp.status().as_u16())
}

#[instrument(skip_all)]
pub async fn check(
    keys: Vec<String>,
    query_per_sec: u32,
    client: Client,
    model: &Option<String>,
) -> anyhow::Result<()> {
    let bar = ProgressBar::new(keys.len() as u64);
    let tasks = keys.into_iter().map(|key| {
        let client = client.clone();
        let bar = bar.clone();
        let model = model.clone();
        async move {
            let resp = match model {
                Some(model) => say_hi(&key, &model, client).await,
                None => list_model(&key, client).await,
            };
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
async fn check_resp(resp: Vec<(String, anyhow::Result<u16>)>) -> anyhow::Result<()> {
    let mut have_banlance_keys = Vec::new();
    let mut ratelimit_keys = Vec::new();
    let mut invalid_keys = Vec::new();
    let mut unknow_error_keys = Vec::new();
    let mut detail = Vec::new();
    detail.push("key, status_code".to_string());
    for (key, resp) in resp.iter() {
        match resp {
            Ok(status_code) => {
                match status_code {
                    200 => have_banlance_keys.push(key),
                    429 => ratelimit_keys.push(key),
                    400 | 401 | 403 => invalid_keys.push(key),
                    _ => unknow_error_keys.push(key),
                };
                detail.push(format!("{key}, {status_code}"));
            }
            Err(err) => {
                tracing::error!("Error: {key}, {err}");
                unknow_error_keys.push(key);
                continue;
            }
        }
    }

    let prefix = "gemini";
    save_to_file(have_banlance_keys, &format!("{prefix}_key")).await?;
    save_to_file(ratelimit_keys, &format!("{prefix}_429_keys")).await?;
    save_to_file(invalid_keys, &format!("{prefix}_invalid_keys")).await?;
    save_to_file(unknow_error_keys, &format!("{prefix}_unknow_err_key")).await?;
    save_to_file(detail, &format!("{prefix}_detail.csv")).await?;

    Ok(())
}
