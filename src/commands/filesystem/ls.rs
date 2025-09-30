use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::env;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct LsCommand;

#[async_trait]
impl BotCommand for LsCommand {
    fn name(&self) -> &str { "ls" }
    fn description(&self) -> &str { "List files and directories in the current location" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".ls [path]" }
    fn examples(&self) -> &'static [&'static str] { &[".ls", ".ls /tmp"] }
    fn aliases(&self) -> &'static [&'static str] { &["dir", "list"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let path_arg = args.next().unwrap_or(".");
        let path = if path_arg == "." {
            env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
        } else {
            Path::new(path_arg).to_path_buf()
        };

        if !path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Path not found: `{}`", path.display()))
                .await?;
            return Ok(());
        }

        if !path.is_dir() {
            http.create_message(msg.channel_id)
                .content(&format!(
                    "ERROR: Path is not a directory: `{}`",
                    path.display()
                ))
                .await?;
            return Ok(());
        }

        match fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                let mut directories = Vec::new();

                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();

                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_dir() {
                                directories.push(format!("**{}**", file_name_str));
                            } else {
                                let size = metadata.len();
                                let size_str = if size < 1024 {
                                    format!("{} B", size)
                                } else if size < 1024 * 1024 {
                                    format!("{:.1} KB", size as f64 / 1024.0)
                                } else {
                                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                                };
                                files.push(format!("{} ({})", file_name_str, size_str));
                            }
                        }
                    }
                }

                directories.sort();
                files.sort();

                let mut output = format!("**Directory listing for:** `{}`\n\n", path.display());

                if !directories.is_empty() {
                    output.push_str("**Directories:**\n");
                    output.push_str(&directories.join("\n"));
                    output.push('\n');
                }

                if !files.is_empty() {
                    output.push_str("**Files:**\n");
                    output.push_str(&files.join("\n"));
                }

                if directories.is_empty() && files.is_empty() {
                    output.push_str("*Empty directory*");
                }

                // Discord has a 2000 character limit
                if output.len() > 1900 {
                    let truncated = &output[..1900];
                    http.create_message(msg.channel_id)
                        .content(&format!("{}\n\n*(truncated)*", truncated))
                        .await?;
                } else {
                    http.create_message(msg.channel_id).content(&output).await?;
                }
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "ERROR: Failed to read directory `{}`: {}",
                        path.display(),
                        e
                    ))
                    .await?;
            }
        }

        Ok(())
    }
}
