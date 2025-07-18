use anyhow::Ok;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    credit_balance: f64,
}

#[instrument]
pub async fn get_balance(key: &str, client: Client) -> anyhow::Result<Response> {
    let url = "https://api.ppinfra.com/v3/user";
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await?;
    tracing::debug!("{:#?}", &resp);
    Ok(resp)
}

#[instrument]
pub async fn total_balance(resp: anyhow::Result<Response>) -> anyhow::Result<f64> {
    let resp = resp?;
    let status_code = resp.status();
    let text = resp.text().await?;
    tracing::debug!("`{status_code}`: `{text}`");
    let info = serde_json::from_str::<BalanceInfo>(&text)
        .map_err(|e| AppError::ResponseError(format!("`{status_code}`: `{text}`, {e:?}")))?;
    let total = info.credit_balance;
    Ok(total)
}
