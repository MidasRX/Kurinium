use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use walkdir::WalkDir;

pub struct SizeCommand;

#[async_trait]
impl BotCommand for SizeCommand {
    fn name(&self) -> &str { "size" }
    fn description(&self) -> &str { "Calculate total size of file or directory" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".size <path>" }
    fn examples(&self) -> &'static [&'static str] { 
        &[".size C:\\Users\\Documents", ".size file.zip"] 
    }
    fn aliases(&self) -> &'static [&'static str] { &["du", "diskusage"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let path_str_owned = args.rest();
        let path_str = path_str_owned.trim();

        if path_str.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a path. Usage: `.size <path>`")
                .await?;
            return Ok(());
        }

        let path = Path::new(path_str);

        if !path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Path not found: `{}`", path_str))
                .await?;
            return Ok(());
        }

        let thinking_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Calculating size of `{}`...", path_str))
            .await?;
        let thinking_message = thinking_msg.model().await?;

        let metadata = fs::metadata(path)?;

        if metadata.is_file() {
            let file_size = metadata.len();
            http.update_message(msg.channel_id, thinking_message.id)
                .content(Some(&format!(
                    "**File Size:** `{}`\n**Size:** {}",
                    path.display(),
                    Self::format_bytes(file_size)
                )))
                .await?;
        } else if metadata.is_dir() {
            match Self::calculate_dir_size(path) {
                Ok(result) => {
                    let mut output = format!("**Directory Size:** `{}`\n\n", path.display());
                    output.push_str(&format!("**Total Size:** {}\n", Self::format_bytes(result.total_size)));
                    output.push_str(&format!("**Files:** {}\n", result.file_count));
                    output.push_str(&format!("**Directories:** {}\n", result.dir_count));
                    
                    if result.error_count > 0 {
                        output.push_str(&format!("\n⚠ **Inaccessible items:** {}", result.error_count));
                    }

                    http.update_message(msg.channel_id, thinking_message.id)
                        .content(Some(&output))
                        .await?;
                }
                Err(e) => {
                    http.update_message(msg.channel_id, thinking_message.id)
                        .content(Some(&format!(
                            "ERROR: Failed to calculate directory size: {}",
                            e
                        )))
                        .await?;
                }
            }
        }

        Ok(())
    }
}

struct SizeResult {
    total_size: u64,
    file_count: usize,
    dir_count: usize,
    error_count: usize,
}

impl SizeCommand {
    fn calculate_dir_size(path: &Path) -> Result<SizeResult> {
        let mut total_size: u64 = 0;
        let mut file_count: usize = 0;
        let mut dir_count: usize = 0;
        let mut error_count: usize = 0;

        for entry in WalkDir::new(path).into_iter() {
            match entry {
                Ok(entry) => {
                    let entry_path = entry.path();
                    
                    if entry_path.is_file() {
                        match fs::metadata(entry_path) {
                            Ok(metadata) => {
                                total_size += metadata.len();
                                file_count += 1;
                            }
                            Err(_) => {
                                error_count += 1;
                            }
                        }
                    } else if entry_path.is_dir() && entry_path != path {
                        dir_count += 1;
                    }
                }
                Err(_) => {
                    error_count += 1;
                }
            }
        }

        Ok(SizeResult {
            total_size,
            file_count,
            dir_count,
            error_count,
        })
    }

    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.2} TB ({} bytes)", bytes as f64 / TB as f64, bytes)
        } else if bytes >= GB {
            format!("{:.2} GB ({} bytes)", bytes as f64 / GB as f64, bytes)
        } else if bytes >= MB {
            format!("{:.2} MB ({} bytes)", bytes as f64 / MB as f64, bytes)
        } else if bytes >= KB {
            format!("{:.2} KB ({} bytes)", bytes as f64 / KB as f64, bytes)
        } else {
            format!("{} bytes", bytes)
        }
    }
}
