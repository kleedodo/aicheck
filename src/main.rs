use std::{path::PathBuf, sync::Arc, time::Duration};

use aicheck::{
    deepseek, gemini, save_to_file,
    siliconflow::{self},
};
use clap::{Parser, ValueEnum};
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{ClientBuilder, StatusCode};
use tokio_stream::wrappers::IntervalStream;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum KeyType {
    Siliconflow,
    Deepseek,
    Gemini,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_parser = check_file_exists)]
    keys_file: PathBuf,
    #[arg(short, long, value_enum)]
    r#type: KeyType,
    #[arg(short, long)]
    num: Option<usize>,
}

fn check_file_exists(file: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(file);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("{file} not exists"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let cli = Cli::parse();
    let contents = tokio::fs::read_to_string(&cli.keys_file).await?;
    let mut have_banlance = Vec::new();
    let mut no_balance = Vec::new();
    let mut unknow = Vec::new();
    let mut results = Vec::new();
    let client = Arc::new(
        ClientBuilder::new()
            .http1_title_case_headers()
            .connect_timeout(Duration::from_secs(30))
            .build()?,
    );
    let bar = ProgressBar::new(contents.lines().count() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    tracing::info!("正在检查...");
    let mut total = 0_f64;
    let keys = contents
        .lines()
        .map(|k| k.trim())
        .filter(|k| !k.is_empty())
        .filter(|k| !k.starts_with("#"))
        .collect::<Vec<_>>();

    let tasks = keys.into_iter().map(|key| {
        let client = client.clone();
        let bar = bar.clone();
        async move {
            let resp = match cli.r#type {
                KeyType::Siliconflow => siliconflow::get_userinfo(key, client).await,
                KeyType::Deepseek => deepseek::get_balance(key, client).await,
                KeyType::Gemini => gemini::say_hi(key, client).await,
            };
            bar.inc(1);
            (key, resp)
        }
    });
    let rate_limit = cli.num.unwrap_or(3);
    let interval_time = 1_f64 / (rate_limit as f64);
    tracing::debug!("interval_time: `{interval_time}`");
    let interval = tokio::time::interval(Duration::from_secs_f64(interval_time));
    let throttled_tasks = IntervalStream::new(interval).zip(stream::iter(tasks));
    let responses = throttled_tasks
        .map(|(_, task)| task)
        .buffer_unordered(rate_limit)
        .collect::<Vec<_>>()
        .await;
    for resp in responses.into_iter() {
        let (key, resp) = resp;
        match cli.r#type {
            KeyType::Siliconflow => match siliconflow::total_balance(resp).await {
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
        }
    }

    save_to_file(&have_banlance, "keys").await?;
    save_to_file(&no_balance, "no_balance_keys").await?;
    save_to_file(&unknow, "401_keys").await?;
    bar.finish();
    tracing::info!("详细：+++++++++++++++++++++++++++++++++++++++++++++");
    for key in results {
        println!("{key}");
    }
    if cli.r#type == KeyType::Siliconflow {
        println!("total: {total}");
    }
    Ok(())
}
