use crate::command_registry::get_registry;
use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

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

        let mut messages: Vec<String> = Vec::new();
        let mut current_msg = String::from("```\n=== COMMAND LIST ===\n\n");

        for category in categories {
            let commands = registry.get_commands_by_category(&category);
            if !commands.is_empty() {
                let mut cat_section = format!("[{}]\n", category.to_uppercase());
                
                for cmd in commands {
                    cat_section.push_str(&format!(".{:<12} {}\n", cmd.name, cmd.description));
                }
                cat_section.push('\n');

                if current_msg.len() + cat_section.len() + 10 > 1900 {
                    current_msg.push_str("```");
                    messages.push(current_msg);
                    current_msg = format!("```\n{}", cat_section);
                } else {
                    current_msg.push_str(&cat_section);
                }
            }
        }

        current_msg.push_str("Use .help <cmd> for details\n```");
        messages.push(current_msg);

        for content in messages {
            http.create_message(msg.channel_id)
                .content(&content)
                .await?;
        }

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

            let examples = metadata.examples.join("\n  ");

            let output = format!(
                "```\n=== {} ===\n\n{}\n\nUsage: {}\nCategory: {}\nAliases: {}\n\nExamples:\n  {}\n```",
                metadata.name.to_uppercase(),
                metadata.description,
                metadata.usage,
                metadata.category,
                aliases,
                examples
            );

            http.create_message(msg.channel_id)
                .content(&output)
                .await?;
        } else {
            http.create_message(msg.channel_id)
                .content(&format!("Command '{}' not found. Use .help to see all.", command_name))
                .await?;
        }

        Ok(())
    }
}
