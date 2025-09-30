use crate::commands::Arguments;
use crate::commands::BotCommand;
use crate::core::auth::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct AuthCommand;

#[async_trait]
impl BotCommand for AuthCommand {
    fn name(&self) -> &str { "auth" }
    fn description(&self) -> &str { "Check authentication status and configuration" }
    fn category(&self) -> &str { "core" }
    fn usage(&self) -> &str { ".auth" }
    fn examples(&self) -> &'static [&'static str] { &[".auth"] }
    fn aliases(&self) -> &'static [&'static str] { &["authstatus", "checkauth"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()> {
        let auth_manager = get_auth_manager();
        let config = auth_manager.get_config();
        let mut fields = Vec::new();

        fields.push(EmbedField {
            name: "General Access".to_string(),
            value: if config.auth_all {
                "Enabled (everyone allowed)".to_string()
            } else {
                "Disabled (authentication required)".to_string()
            },
            inline: false,
        });

        // Role auth
        fields.push(EmbedField {
            name: "Role Authentication".to_string(),
            value: format!(
                "**Status**: {}\n**Allowed Roles**: {}",
                if config.auth_roles {
                    "Enabled"
                } else {
                    "Disabled"
                },
                config.allowed_roles.len()
            ),
            inline: false,
        });

        // User auth
        fields.push(EmbedField {
            name: "User Authentication".to_string(),
            value: format!(
                "**Status**: {}\n**Allowed Users**: {}",
                if config.auth_user {
                    "Enabled"
                } else {
                    "Disabled"
                },
                config.allowed_users.len()
            ),
            inline: false,
        });

        // User status
        let auth_status = auth_manager.get_auth_status(http, msg).await;
        fields.push(EmbedField {
            name: "Your Status".to_string(),
            value: auth_status,
            inline: false,
        });

        let mut embed = EmbedBuilder::new()
            .title("Authentication Status")
            .description("Current authentication configuration and your access status")
            .color(0x0099FF)
            .footer(EmbedFooterBuilder::new("Kurinium Authentication System"));

        for field in fields {
            embed = embed.field(field);
        }

        let embed = embed.build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }
}
