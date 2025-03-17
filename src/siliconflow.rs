use anyhow::Ok;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoResponse {
    code: u32,
    message: String,
    status: bool,
    pub data: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    id: String,
    name: String,
    image: String,
    email: String,
    is_admin: bool,
    pub balance: String,
    status: String,
    introduction: String,
    role: String,
    charge_balance: String,
    total_balance: String,
}
#[instrument]
pub async fn get_userinfo(key: &str, client: Client) -> anyhow::Result<Response> {
    let url = "https://api.siliconflow.cn/v1/user/info";
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await?;
    Ok(resp)
}

pub async fn total_balance(resp: anyhow::Result<Response>) -> anyhow::Result<f64> {
    let resp = resp?;
    let status_code = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::ResponseError(format!("{:?}", e)))?;
    let resp = serde_json::from_str::<UserInfoResponse>(&text)
        .map_err(|_| AppError::ResponseError(format!("`{status_code}`: `{text}`")))?;
    let total = resp.data.total_balance.parse::<f64>()?;
    Ok(total)
}
