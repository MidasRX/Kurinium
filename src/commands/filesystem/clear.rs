use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct ClearCommand;

#[async_trait]
impl BotCommand for ClearCommand {
    fn name(&self) -> &str { "clear" }
    fn description(&self) -> &str { "Clear the chat channel messages" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".clear [number]" }
    fn examples(&self) -> &'static [&'static str] { &[".clear", ".clear 10"] }
    fn aliases(&self) -> &'static [&'static str] { &["cls", "purge"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        let num_str_owned = args.rest();
        let num_str = num_str_owned.trim();
        let mut num_to_delete = 10; // Default

        if !num_str.is_empty() {
            if let Ok(parsed_num) = num_str.parse::<u16>() {
                num_to_delete = parsed_num.min(100); // Limit to 100 messages
            } else {
                http.create_message(msg.channel_id)
                    .content("ERROR: Please provide a valid number. Usage: `.clear [number]`")
                    .await?;
                return Ok(());
            }
        }

        let messages = http
            .channel_messages(msg.channel_id)
            .limit(num_to_delete)
            .await?;

        let messages_list = messages.model().await?;

        if messages_list.is_empty() {
            http.create_message(msg.channel_id)
                .content("INFO: No messages to delete.")
                .await?;
            return Ok(());
        }

        // Delete messages
        let message_ids: Vec<_> = messages_list.iter().map(|m| m.id).collect();

        if message_ids.len() == 1 {
            http.delete_message(msg.channel_id, message_ids[0]).await?;
        } else {
            http.delete_messages(msg.channel_id, &message_ids).await?;
        }

        // Send confirmation message
        http.create_message(msg.channel_id)
            .content(&format!("SUCCESS: Deleted {} messages.\n-# TIPS: You can use .clear <number>\n-# Kurinium: https://github.com/Mikasuru/Kurinium", message_ids.len()))
            .await?;

        Ok(())
    }
}
