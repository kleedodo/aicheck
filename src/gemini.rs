use std::sync::Arc;

use anyhow::Ok;
use reqwest::{Client, Response, StatusCode};

pub async fn get_model_list(key: &str, client: Arc<Client>) -> anyhow::Result<StatusCode> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={key}");
    let resp = client.get(url).send().await?;
    let status = resp.status();
    if resp.status() != StatusCode::OK {
        tracing::error!("{} - {}", status, resp.text().await?);
    }
    Ok(status)
}

pub async fn say_hi(key: &str, client: Arc<Client>) -> anyhow::Result<Response> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-8b:generateContent?key={key}");
    let resp = client
        .post(url)
        .json(&serde_json::json!({
            "contents": [{
                "parts": [{"text": "hi"}]
            }]
        }))
        .send()
        .await?;
    Ok(resp)
}
