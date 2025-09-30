use crate::command_registry::get_registry;
use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::embed::EmbedFooter;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::EmbedBuilder;

pub struct HelpCommand;

#[async_trait]
impl BotCommand for HelpCommand {
    fn name(&self) -> &str { "help" }
    fn description(&self) -> &str { "Show help information for commands" }
    fn category(&self) -> &str { "core" }
    fn usage(&self) -> &str { ".help [command_name]" }
    fn examples(&self) -> &'static [&'static str] { &[".help", ".help ping", ".help upload"] }
    fn aliases(&self) -> &'static [&'static str] { &["h", "commands"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        if args.is_empty() {
            self.show_all_commands(http, msg).await?;
        } else {
            let command_name = args.next().unwrap_or("");
            self.show_command_help(http, msg, command_name).await?;
        }

        Ok(())
    }
}

impl HelpCommand {
    async fn show_all_commands(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let registry = get_registry();
        let categories = registry.get_categories();

        let mut category_fields = Vec::new();

        for category in categories {
            let commands = registry.get_commands_by_category(&category);
            if !commands.is_empty() {
                let command_list = commands
                    .iter()
                    .map(|cmd| format!("**{}** - {}", cmd.name, cmd.description))
                    .collect::<Vec<_>>()
                    .join("\n");

                category_fields.push(EmbedField {
                    name: format!("**{} Commands**", category.to_uppercase()),
                    value: command_list,
                    inline: false,
                });
            }
        }

        let mut embed_builder = EmbedBuilder::new()
            .title("**Command list**")
            .description("Here are all the available kurinium commands:")
            .color(0x0099ff);

        for field in category_fields {
            embed_builder = embed_builder.field(field);
        }

        let embed = embed_builder
            .footer(EmbedFooter {
                text: "Use .help <command> for more information | Kurinium: <https://github.com/Mikasuru/Kurinium>".to_string(),
                icon_url: None,
                proxy_icon_url: None,
            })
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }

    async fn show_command_help(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        command_name: &str,
    ) -> Result<()> {
        let registry = get_registry();
        if let Some(metadata) = registry.get_metadata(command_name) {
            let aliases = if metadata.aliases.is_empty() {
                "None".to_string()
            } else {
                metadata.aliases.join(", ")
            };

            let examples = metadata.examples.join("\n");

            let embed = EmbedBuilder::new()
                .title(format!("**Help for `{}`**", metadata.name))
                .description(metadata.description)
                .color(0x0099ff)
                .field(EmbedField {
                    name: "**Usage**".to_string(),
                    value: metadata.usage,
                    inline: false,
                })
                .field(EmbedField {
                    name: "**Category**".to_string(),
                    value: metadata.category,
                    inline: false,
                })
                .field(EmbedField {
                    name: "**Aliases**".to_string(),
                    value: aliases,
                    inline: false,
                })
                .field(EmbedField {
                    name: "**Examples**".to_string(),
                    value: examples,
                    inline: false,
                })
                .footer(EmbedFooter {
                    text: "For more help, use .help".to_string(),
                    icon_url: None,
                    proxy_icon_url: None,
                })
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;
        } else {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Command `{}` not found. Use `.help` to see all commands.\n-# Kurinium: <https://github.com/Mikasuru/Kurinium>", command_name))
                .await?;
        }

        Ok(())
    }
}
