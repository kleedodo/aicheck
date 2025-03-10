pub mod deepseek;
pub mod errors;
pub mod gemini;
pub mod siliconflow;

pub async fn save_to_file(keys: &[&str], filename: &str) -> anyhow::Result<()> {
    let all_key = keys.join("\n");
    tokio::fs::write(filename, all_key.as_bytes()).await?;
    Ok(())
}
