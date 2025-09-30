use crate::commands::Arguments;
use crate::commands::BotCommand;
use anyhow::Result;
use async_trait::async_trait;
use clipboard_win::{get_clipboard_string, set_clipboard_string};
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct ClipboardCommand;

#[async_trait]
impl BotCommand for ClipboardCommand {
    fn name(&self) -> &str { "clipboard" }
    fn description(&self) -> &str { "Get or set clipboard content" }
    fn category(&self) -> &str { "utility" }
    fn usage(&self) -> &str { ".clipboard <get|set> [text]" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".clipboard get",
            ".clipboard set Kurinium!",
            ".clipboard set \"Kurinium is free on guthib~!\"",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["clip", "cb"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = match args.next() {
            Some(action) => action,
            None => {
                let embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title("Clipboard Manager")
                    .description("Get or set clipboard content")
                    .color(0xFF6B6B)
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Usage".to_string(),
                        value: ".clipboard <get|set> [text]".to_string(),
                        inline: false,
                    })
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Actions".to_string(),
                        value: "**get** - Get current clipboard content\n**set** - Set clipboard content".to_string(),
                        inline: false,
                    })
                    .footer(twilight_util::builder::embed::EmbedFooterBuilder::new("Kurinium Utility Commands"))
                    .build();

                http.create_message(msg.channel_id).embeds(&[embed]).await?;
                return Ok(());
            }
        };

        match action {
            "get" => self.get_clipboard(http, msg).await,
            "set" => {
                let text = args.rest();
                if text.is_empty() {
                    http.create_message(msg.channel_id)
                        .content("**Error**: Please provide text to set on clipboard")
                        .await?;
                    return Ok(());
                }
                self.set_clipboard(http, msg, &text).await
            }
            _ => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "**Error**: Unknown action '{}'. Use 'get' or 'set'",
                        action
                    ))
                    .await?;
                Ok(())
            }
        }
    }
}

impl ClipboardCommand {
    async fn get_clipboard(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        match get_clipboard_string() {
            Ok(content) => {
                if content.is_empty() {
                    http.create_message(msg.channel_id)
                        .content("Clipboard is empty")
                        .await?;
                } else {
                    if content.len() > 1900 {
                        let truncated = format!(
                            "{}...\n\n*(Content truncated, showing first 1900 characters)*",
                            &content[..1900]
                        );

                        let embed = twilight_util::builder::embed::EmbedBuilder::new()
                            .title("Clipboard Content")
                            .description("```\n".to_string() + &truncated + "\n```")
                            .color(0x00D4AA)
                            .footer(twilight_util::builder::embed::EmbedFooterBuilder::new(
                                &format!("Length: {} characters (truncated)", content.len()),
                            ))
                            .build();

                        http.create_message(msg.channel_id).embeds(&[embed]).await?;
                    } else {
                        let embed = twilight_util::builder::embed::EmbedBuilder::new()
                            .title("Clipboard Content")
                            .description("```\n".to_string() + &content + "\n```")
                            .color(0x00D4AA)
                            .footer(twilight_util::builder::embed::EmbedFooterBuilder::new(
                                &format!("Length: {} characters", content.len()),
                            ))
                            .build();

                        http.create_message(msg.channel_id).embeds(&[embed]).await?;
                    }
                }
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to get clipboard content: {}", e))
                    .await?;
            }
        }

        Ok(())
    }

    async fn set_clipboard(&self, http: &Arc<HttpClient>, msg: &Message, text: &str) -> Result<()> {
        match set_clipboard_string(text) {
            Ok(_) => {
                let preview = if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    text.to_string()
                };

                let embed = twilight_util::builder::embed::EmbedBuilder::new()
                    .title("Clipboard Set")
                    .description("Clipboard content has been updated successfully")
                    .color(0x32CD32)
                    .field(twilight_model::channel::message::embed::EmbedField {
                        name: "Preview".to_string(),
                        value: format!("```\n{}\n```", preview),
                        inline: false,
                    })
                    .footer(twilight_util::builder::embed::EmbedFooterBuilder::new(
                        &format!("Length: {} characters", text.len()),
                    ))
                    .build();

                http.create_message(msg.channel_id).embeds(&[embed]).await?;
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to set clipboard content: {}", e))
                    .await?;
            }
        }

        Ok(())
    }
}
