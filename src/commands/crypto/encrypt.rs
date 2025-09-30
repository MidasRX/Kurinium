use crate::commands::*;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rand::RngCore;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct EncryptCommand;

#[async_trait]
impl BotCommand for EncryptCommand {
    fn name(&self) -> &'static str { "encrypt" }
    fn description(&self) -> &str { "Encrypt any file using AES-256-GCM" }
    fn category(&self) -> &str { "crypto" }
    fn usage(&self) -> &str { ".encrypt \"file_path\"" }
    fn examples(&self) -> &'static [&'static str] { &[".encrypt test.txt", ".encrypt \"my file.txt\""] }
    fn aliases(&self) -> &'static [&'static str] { &["enc"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let rest = args.rest();
        let file_path = rest.trim();

        if file_path.is_empty() {
            let embed = EmbedBuilder::new()
                .title("Missing File Path")
                .description(
                    "Please provide a file path to encrypt.\n\n**Usage:** `.encrypt \"file_path\"`",
                )
                .color(0xFF0000)
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;

            return Ok(());
        }

        if !Path::new(file_path).exists() {
            let embed = EmbedBuilder::new()
                .title("File Not Found")
                .description(format!("The file `{}` does not exist.", file_path))
                .color(0xFF0000)
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;

            return Ok(());
        }

        // Random encryption key and nonce
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut nonce);

        let file_content =
            fs::read(file_path).with_context(|| format!("Failed to read file: {}", file_path))?;

        let cipher = Aes256Gcm::new(&key.into());
        let encrypted_data = cipher
            .encrypt(&nonce.into(), &file_content[..])
            .map_err(|e| anyhow::anyhow!("Failed to encrypt file: {}", e))?;

        let path = Path::new(file_path);
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let output_path = format!("{}.encrypted", filename);

        let mut output = Vec::new();
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&encrypted_data);
        fs::write(&output_path, output)
            .with_context(|| format!("Failed to write encrypted file: {}", output_path))?;

        let key_hex = hex::encode(&key);

        let embed = EmbedBuilder::new()
            .title("File Encrypted Successfully")
            .description(format!(
                "**Original:** `{}`\n**Encrypted:** `{}`\n**Size:** {} bytes",
                filename,
                output_path,
                encrypted_data.len()
            ))
            .field(EmbedField {
                name: "Encryption Key".to_string(),
                value: format!("```\n{}```", key_hex),
                inline: false,
            })
            .field(EmbedField {
                name: "Important".to_string(),
                value: "Save this key securely! You'll need it to decrypt the file.".to_string(),
                inline: false,
            })
            .color(0x00FF00)
            .footer(EmbedFooterBuilder::new("AES-256-GCM Encryption"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }
}
