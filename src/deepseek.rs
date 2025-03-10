use std::sync::Arc;

use anyhow::Ok;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBalanceRespone {
    pub is_available: bool,
    pub balance_infos: Vec<BalanceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub currency: String,
    pub total_balance: String,
    pub granted_balance: String,
    pub topped_up_balance: String,
}

#[instrument]
pub async fn get_balance(key: &str, client: Arc<Client>) -> anyhow::Result<Response> {
    let url = "https://api.deepseek.com/user/balance";
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await?;
    tracing::debug!("{:#?}", &resp);
    Ok(resp)
}

pub async fn total_balance(resp: anyhow::Result<Response>) -> anyhow::Result<f64> {
    let resp = resp?;
    let status_code = resp.status();
    let text = resp.text().await?;
    let info = serde_json::from_str::<UserBalanceRespone>(&text)
        .map_err(|_| AppError::ResponseError(format!("`{status_code}`: `{text}`")))?;
    let banlance_info = &info.balance_infos[0];
    let total = banlance_info.total_balance.parse::<f64>()?;
    Ok(total)
}
