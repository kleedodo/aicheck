use clap::ValueEnum;
use indicatif::ProgressBar;
use reqwest::{Response, StatusCode};

pub mod deepseek;
pub mod errors;
pub mod gemini;
pub mod ppinfra;
pub mod siliconflow;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum KeyType {
    Siliconflow,
    Deepseek,
    Gemini,
    Ppinfra,
}
pub async fn save_to_file(keys: &[&str], filename: &str) -> anyhow::Result<()> {
    let mut all_key = keys.join("\n");
    if !all_key.is_empty() {
        all_key.push('\n');
    }
    tokio::fs::write(filename, all_key.as_bytes()).await?;
    Ok(())
}

pub async fn check_resp(
    responses: Vec<(&str, anyhow::Result<Response>)>,
    channel_type: KeyType,
    bar: ProgressBar,
) -> anyhow::Result<()> {
    let mut total = 0_f64;
    let mut total_pro = 0_f64;
    let mut have_banlance = Vec::new();
    let mut no_balance = Vec::new();
    let mut unknow = Vec::new();
    let mut results = Vec::new();
    let mut pro_banlace = Vec::new();
    for resp in responses.into_iter() {
        let (key, resp) = resp;
        match channel_type {
            KeyType::Siliconflow => match siliconflow::userinfo(resp).await {
                Ok(user) => {
                    let charge_balance = user.charge_balance.parse::<f64>().unwrap_or_default();
                    let total_balance = user.total_balance.parse::<f64>().unwrap_or_default();
                    if charge_balance > 0_f64 {
                        pro_banlace.push(key);
                        total_pro += charge_balance;
                    }
                    if total_balance > 0_f64 {
                        total += total_balance;
                        have_banlance.push(key);
                    } else {
                        no_balance.push(key);
                    }
                    results.push(format!("{key}, {total_balance}, {charge_balance}"));
                }
                Err(err) => {
                    unknow.push(key);
                    tracing::debug!("`{key}`: `{err}`")
                }
            },
            KeyType::Deepseek => match deepseek::total_balance(resp).await {
                Ok(balance) => {
                    if balance > 0_f64 {
                        total += balance;
                        have_banlance.push(key);
                    } else {
                        no_balance.push(key);
                    }
                    results.push(format!("{key}, {balance}"));
                }
                Err(err) => {
                    unknow.push(key);
                    tracing::debug!("`{key}`: `{err}`")
                }
            },
            KeyType::Gemini => match resp {
                Ok(resp) => {
                    match resp.status() {
                        StatusCode::OK => {
                            have_banlance.push(key);
                        }
                        StatusCode::TOO_MANY_REQUESTS => {
                            no_balance.push(key);
                        }
                        _ => {
                            unknow.push(key);
                        }
                    }
                    let text = resp.text().await;
                    tracing::debug!("`{key}`, `{:?}`", text);
                }
                Err(err) => {
                    unknow.push(key);
                    tracing::debug!("`{key}`: `{err}`")
                }
            },
            KeyType::Ppinfra => match ppinfra::total_balance(resp).await {
                Ok(balance) => {
                    if balance > 0_f64 {
                        total += balance;
                        have_banlance.push(key);
                    } else {
                        no_balance.push(key);
                    }
                    results.push(format!("{key}, {balance}"));
                }
                Err(err) => {
                    unknow.push(key);
                    tracing::debug!("`{key}`: `{err}`")
                }
            },
        }
    }

    save_to_file(&have_banlance, "keys").await?;
    save_to_file(&no_balance, "no_balance_keys").await?;
    save_to_file(&pro_banlace, "pro_keys").await?;
    save_to_file(&unknow, "401_keys").await?;
    bar.finish();
    tracing::info!("详细：+++++++++++++++++++++++++++++++++++++++++++++");
    for key in results {
        println!("{key}");
    }
    match channel_type {
        KeyType::Siliconflow => {
            println!("total: {total}, total pro: {total_pro}");
        }
        KeyType::Deepseek | KeyType::Ppinfra => {
            println!("total: {total}");
        }
        _ => {}
    }
    Ok(())
}
