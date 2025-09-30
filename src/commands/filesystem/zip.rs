use crate::commands::Arguments;
use crate::commands::BotCommand;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

pub struct ZipCommand;

#[async_trait]
impl BotCommand for ZipCommand {
    fn name(&self) -> &str { "zip" }
    fn description(&self) -> &str { "Compress a file or directory to ZIP archive" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".zip <file_or_directory>" }
    fn examples(&self) -> &'static [&'static str] { &[".zip C:\\Users\\Kurinium\\Desktop\\Kurinium", ".zip Kurinium", ".zip \"Kurinium is free\"", ".zip kurinium.txt"] }
    fn aliases(&self) -> &'static [&'static str] { &["compress", "archive"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let target = args.rest();
        if target.is_empty() {
            let embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title("Zip Command")
                    .description("Compress files or directories to ZIP archives")
                    .color(0xFF6B6B)
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Usage".to_string(),
                        value: ".zip <file_or_directory>".to_string(),
                        inline: false,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Examples".to_string(),
                        value: "**Absolute path:** `.zip C:\\Users\\ASUS\\Desktop\\Kurinium`\n**Relative path:** `.zip Data`\n**With spaces:** `.zip \"My Folder\"`\n**Single file:** `.zip document.txt`".to_string(),
                        inline: false,
                    })
                    .footer(twilight_util::builder::embed::EmbedFooterBuilder::new("Kurinium Filesystem Commands"))
                    .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;
            return Ok(());
        }

        self.zip_item(http, msg, &target).await
    }
}

impl ZipCommand {
    const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer for streaming
    async fn zip_item(&self, http: &Arc<HttpClient>, msg: &Message, target: &str) -> Result<()> {
        let path = PathBuf::from(target);
        if !path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("Path '{}' does not exist", target))
                .await?;
            return Ok(());
        }

        let zip_filename = if path.is_dir() {
            format!(
                "{}.zip",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        } else {
            format!(
                "{}.zip",
                path.file_stem().unwrap_or_default().to_string_lossy()
            )
        };

        if Path::new(&zip_filename).exists() {
            http.create_message(msg.channel_id)
                .content(&format!("File '{}' already exists", zip_filename))
                .await?;
            return Ok(());
        }

        let _thinking_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Compressing '{}'...", target))
            .await?;

        let result = if path.is_file() {
            self.zip_file(&path, &zip_filename)
        } else {
            self.zip_directory(&path, &zip_filename)
        };

        match result {
            Ok(size) => {
                // Get original size for compression ratio
                let original_size = self.get_total_size(&path)?;

                let ratio = if original_size > 0 {
                    if size >= original_size {
                        0.0 // No compression or compression increased size
                    } else {
                        ((original_size - size) as f64 / original_size as f64) * 100.0
                    }
                } else {
                    0.0
                };

                let embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title("Compression Complete")
                    .description(&format!("Successfully compressed '{}'", target))
                    .color(0x2ECC71)
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Output File".to_string(),
                        value: zip_filename,
                        inline: true,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Original Size".to_string(),
                        value: self.format_bytes(original_size),
                        inline: true,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Compressed Size".to_string(),
                        value: self.format_bytes(size),
                        inline: true,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Compression Ratio".to_string(),
                        value: format!("{:.1}%", ratio),
                        inline: true,
                    })
                    .footer(twilight_util::builder::embed::EmbedFooterBuilder::new(
                        "Kurinium Filesystem Commands",
                    ))
                    .build();

                http.create_message(msg.channel_id).embeds(&[embed]).await?;
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to compress '{}': {}", target, e))
                    .await?;
            }
        }

        Ok(())
    }

    fn zip_file(&self, file_path: &Path, output_path: &str) -> Result<u64> {
        let file = fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);

        let filename = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy();

        zip.start_file(filename.as_ref(), options)?;

        let mut reader = BufReader::new(fs::File::open(file_path)?);
        let mut buffer = [0; Self::BUFFER_SIZE];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            zip.write_all(&buffer[..bytes_read])?;
        }

        zip.finish()?;

        Ok(fs::metadata(output_path)?.len())
    }

    fn zip_directory(&self, dir_path: &Path, output_path: &str) -> Result<u64> {
        let file = fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(file);

        let base_dir_name = dir_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?
            .to_string_lossy();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(dir_path)?;

            if path.is_file() {
                let zip_path = Path::new(base_dir_name.as_ref()).join(relative_path);
                let zip_path_str = zip_path.to_string_lossy().replace('\\', "/");

                let options = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o644);

                zip.start_file(&zip_path_str, options)?;

                let mut reader = BufReader::new(fs::File::open(path)?);
                let mut buffer = [0; Self::BUFFER_SIZE];

                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    zip.write_all(&buffer[..bytes_read])?;
                }
            } else if path.is_dir() && relative_path != Path::new("") {
                // Add directory entry
                let zip_path = Path::new(base_dir_name.as_ref()).join(relative_path);
                let zip_path_str = format!("{}/", zip_path.to_string_lossy().replace('\\', "/"));

                let options = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o755);

                zip.add_directory(&zip_path_str, options)?;
            }
        }

        zip.finish()?;

        Ok(fs::metadata(output_path)?.len())
    }

    fn get_total_size(&self, path: &Path) -> Result<u64> {
        if path.is_file() {
            Ok(fs::metadata(path)?.len())
        } else {
            let mut total_size = 0;
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    total_size += fs::metadata(entry.path())?.len();
                }
            }
            Ok(total_size)
        }
    }

    fn format_bytes(&self, bytes: u64) -> String {
        crate::utils::formatting::format_memory_size(bytes)
    }
}
