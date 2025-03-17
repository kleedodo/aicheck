use anyhow::Ok;
use reqwest::{Client, Response};

pub async fn say_hi(key: &str, client: Client) -> anyhow::Result<Response> {
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
