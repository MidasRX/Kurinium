use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct MkdirCommand;

#[async_trait]
impl BotCommand for MkdirCommand {
    fn name(&self) -> &str { "mkdir" }
    fn description(&self) -> &str { "Create a new directory" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".mkdir <directory_path>" }
    fn examples(&self) -> &'static [&'static str] { &[".mkdir new_folder"] }
    fn aliases(&self) -> &'static [&'static str] { &["md"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let dir_path_owned = args.rest();
        let dir_path = dir_path_owned.trim();

        if dir_path.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a directory path. Usage: `.mkdir <directory_path>`")
                .await?;
            return Ok(());
        }

        let path = Path::new(dir_path);

        if path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Directory already exists: `{}`", dir_path))
                .await?;
            return Ok(());
        }

        match fs::create_dir_all(path) {
            Ok(_) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "SUCCESS: **Directory created successfully:** `{}`",
                        dir_path
                    ))
                    .await?;
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "ERROR: Failed to create directory `{}`: {}",
                        dir_path, e
                    ))
                    .await?;
            }
        }

        Ok(())
    }
}
