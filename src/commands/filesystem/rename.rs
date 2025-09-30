use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct RenameCommand;

#[async_trait]
impl BotCommand for RenameCommand {
    fn name(&self) -> &str { "rename" }
    fn description(&self) -> &str { "Rename a file or directory" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".rename <old_path> <new_path>" }
    fn examples(&self) -> &'static [&'static str] { &[".rename old.txt new.txt", ".rename \"old file.txt\" \"new file.txt\""] }
    fn aliases(&self) -> &'static [&'static str] { &["mv", "ren"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let parsed_args = Arguments::parse_quoted_args(&args.rest());

        if parsed_args.len() < 2 {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide both old and new paths. Usage: `.rename <old_path> <new_path>`\nFor paths with spaces use quotes: `.rename \"old file.txt\" \"new file.txt\"`")
                .await?;
            return Ok(());
        }

        let old_path = &parsed_args[0];
        let new_path = &parsed_args[1];

        let old_path_obj = Path::new(old_path);
        let new_path_obj = Path::new(new_path);

        if !old_path_obj.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Source path not found: `{}`", old_path))
                .await?;
            return Ok(());
        }

        if new_path_obj.exists() {
            http.create_message(msg.channel_id)
                .content(&format!(
                    "ERROR: Destination path already exists: `{}`",
                    new_path
                ))
                .await?;
            return Ok(());
        }

        match fs::rename(old_path_obj, new_path_obj) {
            Ok(_) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "SUCCESS: **Renamed successfully:**\n**From:** `{}`\n**To:** `{}`",
                        old_path, new_path
                    ))
                    .await?;
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "ERROR: Failed to rename `{}` to `{}`: {}",
                        old_path, new_path, e
                    ))
                    .await?;
            }
        }

        Ok(())
    }
}
