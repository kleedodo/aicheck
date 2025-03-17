use std::{path::PathBuf, time::Duration};

use aicheck::{
    check_resp, deepseek, gemini, ppinfra,
    siliconflow::{self},
    KeyType,
};
use clap::Parser;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::ClientBuilder;
use tokio_stream::wrappers::IntervalStream;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let client = ClientBuilder::new()
        .http1_title_case_headers()
        .connect_timeout(Duration::from_secs(30))
        .build()?;
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
                KeyType::Ppinfra => ppinfra::get_balance(key, client).await,
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
    check_resp(responses, cli.r#type, bar).await?;
    Ok(())
}
