use std::ops::Deref;

pub mod deepseek;
pub mod gemini;
pub mod openrouter;
pub mod ppinfra;
pub mod siliconflow;

pub async fn save_to_file<I, S>(keys: I, filename: &str) -> anyhow::Result<()>
where
    I: Deref<Target = [S]>,
    S: AsRef<str>,
{
    if keys.is_empty() {
        return Ok(());
    }
    let all_key = keys.iter().map(|k| k.as_ref()).collect::<Vec<&str>>();
    let mut all_key = all_key.join("\n");
    all_key.push('\n');
    tokio::fs::write(filename, all_key.as_bytes()).await?;
    Ok(())
}
