use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct CatCommand;

#[async_trait]
impl BotCommand for CatCommand {
    fn name(&self) -> &str { "cat" }
    fn description(&self) -> &str { "Display contents of a file" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".cat <file_path>" }
    fn examples(&self) -> &'static [&'static str] { &[".cat config.txt", ".cat /path/to/file.log"] }
    fn aliases(&self) -> &'static [&'static str] { &["read"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let file_path_owned = args.rest();
        let file_path = file_path_owned.trim();

        if file_path.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a file path. Usage: `.cat <file_path>`")
                .await?;
            return Ok(());
        }

        let path = Path::new(file_path);

        if !path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: File not found: `{}`", file_path))
                .await?;
            return Ok(());
        }

        if !path.is_file() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Path is not a file: `{}`", file_path))
                .await?;
            return Ok(());
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                // Discord has a 2000 character limit for messages
                if content.len() > 1900 {
                    let truncated = &content[..1900];
                    http.create_message(msg.channel_id)
                        .content(&format!(
                            "**File contents (truncated):**\n```\n{}\n```",
                            truncated
                        ))
                        .await?;
                } else {
                    http.create_message(msg.channel_id)
                        .content(&format!("**File contents:**\n```\n{}\n```", content))
                        .await?;
                }
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "ERROR: Failed to read file `{}`: {}",
                        file_path, e
                    ))
                    .await?;
            }
        }

        Ok(())
    }
}
