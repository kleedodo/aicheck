use std::{path::PathBuf, time::Duration};

use aicheck::{
    deepseek,
    gemini::check,
    openrouter, ppinfra,
    siliconflow::{self},
};
use clap::{Parser, Subcommand};
use reqwest::ClientBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_parser = check_file_exists)]
    keys_file: PathBuf,
    #[arg(short, long)]
    query_per_sec: Option<usize>,
    #[command(subcommand)]
    command: Commands,
}

#[non_exhaustive]
#[derive(Debug, Subcommand)]
enum Commands {
    Siliconflow,
    Deepseek,
    Gemini {
        #[arg(short, long)]
        model: Option<String>,
    },
    Ppinfra,
    OpenRouter,
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
    let query_per_sec = cli.query_per_sec.unwrap_or(3);
    let client = ClientBuilder::new()
        .http1_title_case_headers()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(query_per_sec)
        .build()?;
    tracing::info!("正在检查...");
    let keys = contents
        .lines()
        .map(|k| k.trim())
        .filter(|k| !k.is_empty())
        .filter(|k| !k.starts_with("#"))
        .map(String::from)
        .collect::<Vec<_>>();
    match &cli.command {
        Commands::Siliconflow => siliconflow::check(keys, query_per_sec, client).await?,
        Commands::Deepseek => deepseek::check(keys, query_per_sec, client).await?,
        Commands::Gemini { model } => check(keys, query_per_sec, client, model).await?,
        Commands::Ppinfra => ppinfra::check(keys, query_per_sec, client).await?,
        Commands::OpenRouter => openrouter::check(keys, query_per_sec, client).await?,
    };

    Ok(())
}
