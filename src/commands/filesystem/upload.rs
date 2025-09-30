use crate::commands::*;
use crate::config::Config;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct UploadCommand;

#[async_trait]
impl BotCommand for UploadCommand {
    fn name(&self) -> &str { "upload" }
    fn description(&self) -> &str { "Upload a file to Discord or save Discord attachments to local filesystem" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".upload <file_path> | .upload (with attachment) | .upload save <filename>" }
    fn examples(&self) -> &'static [&'static str] { &[".upload kurinium.pdf", ".upload save kurinium_file.zip"] }
    fn aliases(&self) -> &'static [&'static str] { &["up"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        if !msg.attachments.is_empty() {
            return self.handle_attachment_save(http, msg, args).await;
        }

        let args_string = args.rest();
        let mut parts = args_string.trim().split_whitespace();
        let first_arg = parts.next().unwrap_or("");

        if first_arg == "save" {
            let filename = parts.collect::<Vec<_>>().join(" ");
            if filename.is_empty() {
                http.create_message(msg.channel_id)
                    .content("ERROR: Please provide a filename. Usage: `.upload save <filename>`")
                    .await?;
                return Ok(());
            }
            return self.handle_attachment_download(http, msg, &filename).await;
        }

        if first_arg.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a file path, attach a file, or use `.upload save <filename>`. Usage: `.upload <file_path>`")
                .await?;
            return Ok(());
        }

        let file_path = args_string.trim();

        self.handle_file_upload(http, msg, file_path).await
    }
}

impl UploadCommand {
    async fn handle_attachment_save(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        args: Arguments,
    ) -> Result<()> {
        let attachment = &msg.attachments[0]; // Get first attachment
        let filename_owned = args.rest();
        let filename = filename_owned.trim();

        let save_filename = if filename.is_empty() {
            &attachment.filename
        } else {
            filename
        };

        let response_msg = http
            .create_message(msg.channel_id)
            .content(&format!(
                "Downloading attachment `{}` as `{}`...",
                attachment.filename, save_filename
            ))
            .await?;
        let response_message = response_msg.model().await?;

        let client = reqwest::Client::new();
        match client.get(&attachment.url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let bytes = resp.bytes().await?;

                    match fs::write(save_filename, &bytes) {
                        Ok(_) => {
                            let file_size_mb = bytes.len() as f64 / (1024.0 * 1024.0);
                            http.update_message(msg.channel_id, response_message.id)
                                .content(Some(&format!("SUCCESS: **Attachment saved successfully!**\n**Filename:** `{}`\n**Size:** {:.2} MB", save_filename, file_size_mb)))
                                .await?;
                        }
                        Err(e) => {
                            http.update_message(msg.channel_id, response_message.id)
                                .content(Some(&format!("ERROR: Failed to save attachment: {}", e)))
                                .await?;
                        }
                    }
                } else {
                    http.update_message(msg.channel_id, response_message.id)
                        .content(Some(&format!(
                            "ERROR: Failed to download attachment. Status: {}",
                            resp.status()
                        )))
                        .await?;
                }
            }
            Err(e) => {
                http.update_message(msg.channel_id, response_message.id)
                    .content(Some(&format!(
                        "ERROR: Failed to download attachment: {}",
                        e
                    )))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_attachment_download(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        _filename: &str,
    ) -> Result<()> {
        http.create_message(msg.channel_id)
            .content("INFO: Attachment download from referenced messages not yet implemented.\nPlease attach a file directly to save it.\n-# Kurinium: <https://github.com/Mikasuru/Kurinium>")
            .await?;
        Ok(())
    }

    // Handle uploading local files to Discord
    async fn handle_file_upload(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        file_path: &str,
    ) -> Result<()> {
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

        // Get file metadata
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // Check Discord's file size limit
        let max_size = Config::get_max_bfilesize() as u64;
        if file_size > max_size {
            http.create_message(msg.channel_id)
                .content(&format!(
                    "ERROR: File is too large. Discord has a {:.0}MB limit. File size: {:.2} MB",
                    Config::MAX_FILE_SIZE_MB,
                    file_size as f64 / (1024.0 * 1024.0)
                ))
                .await?;
            return Ok(());
        }

        let response_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Uploading file `{}`...", file_path))
            .await?;
        let response_message = response_msg.model().await?;
        let _file_content = fs::read(path)?;

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("uploaded_file");

        let file_size_mb = file_size as f64 / (1024.0 * 1024.0);

        match http.create_message(msg.channel_id)
            .content(&format!("SUCCESS: **File ready for upload:**\n**Filename:** `{}`\n**Size:** {:.2} MB\n\n*Note: Actual file upload requires proper attachment handling*", filename, file_size_mb))
            .await {
                Ok(_) => {
                    // Delete the "uploading" message
                    let _ = http.delete_message(msg.channel_id, response_message.id).await;
                }
                Err(e) => {
                    http.update_message(msg.channel_id, response_message.id)
                        .content(Some(&format!("ERROR: Failed to upload file: {}", e)))
                        .await?;
                }
            }

        Ok(())
    }
}
