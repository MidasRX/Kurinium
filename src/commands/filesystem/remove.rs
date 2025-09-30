use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct RemoveCommand;

#[async_trait]
impl BotCommand for RemoveCommand {
    fn name(&self) -> &str { "remove" }
    fn description(&self) -> &str { "Remove a file or directory" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".remove <path>" }
    fn examples(&self) -> &'static [&'static str] { &[".remove old_file.txt", ".remove /tmp/folder"] }
    fn aliases(&self) -> &'static [&'static str] { &["rm", "del"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let path_str_owned = args.rest();
        let path_str = path_str_owned.trim();

        if path_str.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a path. Usage: `.remove <path>`")
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

        if path.is_file() {
            match fs::remove_file(path) {
                Ok(_) => {
                    http.create_message(msg.channel_id)
                        .content(&format!(
                            "SUCCESS: **File removed successfully:** `{}`",
                            path_str
                        ))
                        .await?;
                }
                Err(e) => {
                    http.create_message(msg.channel_id)
                        .content(&format!(
                            "ERROR: Failed to remove file `{}`: {}",
                            path_str, e
                        ))
                        .await?;
                }
            }
        } else if path.is_dir() {
            match fs::remove_dir_all(path) {
                Ok(_) => {
                    http.create_message(msg.channel_id)
                        .content(&format!(
                            "SUCCESS: **Directory removed successfully:** `{}`",
                            path_str
                        ))
                        .await?;
                }
                Err(e) => {
                    http.create_message(msg.channel_id)
                        .content(&format!(
                            "ERROR: Failed to remove directory `{}`: {}",
                            path_str, e
                        ))
                        .await?;
                }
            }
        } else {
            http.create_message(msg.channel_id)
                .content(&format!(
                    "ERROR: Path is neither a file nor a directory: `{}`",
                    path_str
                ))
                .await?;
        }

        Ok(())
    }
}
