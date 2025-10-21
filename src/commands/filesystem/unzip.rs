use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use zip::ZipArchive;

pub struct UnzipCommand;

#[async_trait]
impl BotCommand for UnzipCommand {
    fn name(&self) -> &str { "unzip" }
    fn description(&self) -> &str { "Extract ZIP archive to current directory" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".unzip <file.zip> [destination]" }
    fn examples(&self) -> &'static [&'static str] { 
        &[".unzip archive.zip", ".unzip data.zip ./extracted"] 
    }
    fn aliases(&self) -> &'static [&'static str] { &["extract"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, mut args: Arguments) -> Result<()> {
        let zip_path = args.next().unwrap_or("").to_string();
        let dest_path_owned = args.rest();
        let dest_path = dest_path_owned.trim();

        if zip_path.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a ZIP file. Usage: `.unzip <file.zip> [destination]`")
                .await?;
            return Ok(());
        }

        let path = Path::new(&zip_path);

        if !path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: File not found: `{}`", zip_path))
                .await?;
            return Ok(());
        }

        if !path.is_file() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Path is not a file: `{}`", zip_path))
                .await?;
            return Ok(());
        }

        let destination = if dest_path.is_empty() {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            PathBuf::from(dest_path)
        };

        let thinking_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Extracting `{}`...", zip_path))
            .await?;
        let thinking_message = thinking_msg.model().await?;

        match Self::extract_zip(path, &destination) {
            Ok((file_count, total_size)) => {
                let embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title("Extraction Complete")
                    .description(&format!("Successfully extracted `{}`", zip_path))
                    .color(0x2ECC71)
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Destination".to_string(),
                        value: format!("`{}`", destination.display()),
                        inline: false,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Files Extracted".to_string(),
                        value: file_count.to_string(),
                        inline: true,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Total Size".to_string(),
                        value: Self::format_bytes(total_size),
                        inline: true,
                    })
                    .footer(twilight_util::builder::embed::EmbedFooterBuilder::new(
                        "Kurinium Filesystem Commands",
                    ))
                    .build();

                http.update_message(msg.channel_id, thinking_message.id)
                    .embeds(Some(&[embed]))
                    .await?;
            }
            Err(e) => {
                http.update_message(msg.channel_id, thinking_message.id)
                    .content(Some(&format!("ERROR: Failed to extract `{}`: {}", zip_path, e)))
                    .await?;
            }
        }

        Ok(())
    }
}

impl UnzipCommand {
    fn extract_zip(zip_path: &Path, destination: &Path) -> Result<(usize, u64)> {
        let file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut file_count = 0;
        let mut total_size = 0u64;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => destination.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;

                total_size += file.size();
                file_count += 1;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok((file_count, total_size))
    }

    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}
