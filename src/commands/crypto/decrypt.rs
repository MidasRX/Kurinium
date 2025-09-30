use crate::commands::*;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct DecryptCommand;

#[async_trait]
impl BotCommand for DecryptCommand {
    fn name(&self) -> &'static str { "decrypt" }
    fn description(&self) -> &str { "Decrypt a file using AES-256-GCM" }
    fn category(&self) -> &str { "crypto" }
    fn usage(&self) -> &str { ".decrypt \"encrypted_file\" <encryption_key>" }
    fn examples(&self) -> &'static [&'static str] { &[".decrypt file.encrypted a1b2c3d4..."] }
    fn aliases(&self) -> &'static [&'static str] { &["dec"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let rest = args.rest();
        let parts: Vec<&str> = rest.split_whitespace().collect();

        if parts.len() < 2 {
            let embed = EmbedBuilder::new()
                .title("Missing Arguments")
                .description("Please provide the encrypted file path and decryption key.\n\n**Usage:** `.decrypt \"encrypted_file\" <encryption_key>`")
                .color(0xFF0000)
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;

            return Ok(());
        }

        let file_path = parts[0];
        let key_hex = parts[1..].join(" ");

        if !Path::new(file_path).exists() {
            let embed = EmbedBuilder::new()
                .title("File Not Found")
                .description(format!("The file `{}` does not exist.", file_path))
                .color(0xFF0000)
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;

            return Ok(());
        }

        let key = hex::decode(&key_hex).context("Invalid encryption key format")?;

        if key.len() != 32 {
            let embed = EmbedBuilder::new()
                .title("Invalid Key Length")
                .description("The encryption key must be 64 characters (32 bytes) long.")
                .color(0xFF0000)
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;

            return Ok(());
        }

        let encrypted_data = fs::read(file_path)
            .with_context(|| format!("Failed to read encrypted file: {}", file_path))?;

        if encrypted_data.len() < 12 {
            let embed = EmbedBuilder::new()
                .title("Invalid File Format")
                .description("The encrypted file appears to be corrupted or invalid.")
                .color(0xFF0000)
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;

            return Ok(());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(nonce_bytes);

        let key_array: [u8; 32] = key.try_into().expect("Invalid key length");
        let cipher = Aes256Gcm::new(&key_array.into());
        let decrypted_data = cipher.decrypt(&nonce.into(), ciphertext).map_err(|e| {
            anyhow::anyhow!(
                "Failed to decrypt file. The key may be incorrect or the file may be corrupted: {}",
                e
            )
        })?;

        let path = Path::new(file_path);
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let base_name = filename.strip_suffix(".encrypted").unwrap_or(&filename);
        let output_path = format!("{}.decrypted", base_name);

        let data_len = decrypted_data.len();

        fs::write(&output_path, decrypted_data)
            .with_context(|| format!("Failed to write decrypted file: {}", output_path))?;

        let embed = EmbedBuilder::new()
            .title("File Decrypted Successfully")
            .description(format!(
                "**Encrypted:** `{}`\n**Decrypted:** `{}`\n**Size:** {} bytes",
                filename, output_path, data_len
            ))
            .color(0x00FF00)
            .footer(EmbedFooterBuilder::new("AES-256-GCM Decryption"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }
}
