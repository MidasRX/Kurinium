use crate::commands::Arguments;
use crate::commands::BotCommand;
use crate::core::process::{format_cpu_usage, format_memory_size, ProcessManager};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::Message;
use twilight_model::http::attachment::Attachment;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct ProcessCommand;

#[async_trait]
impl BotCommand for ProcessCommand {
    fn name(&self) -> &str { "process" }
    fn description(&self) -> &str { "Manage system processes (list, kill, info, installed)" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".process <list|kill|info|installed> [pid|name]" }
    fn examples(&self) -> &'static [&'static str] { &[".process list", ".process kill 1234", ".process info chrome.exe", ".process info 1234", ".process installed"] }
    fn aliases(&self) -> &'static [&'static str] { &["ps", "proc"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = match args.next() {
            Some(action) => action,
            None => {
                let embed = EmbedBuilder::new()
                    .title("Process Management")
                    .description("Manage system processes")
                    .color(0xFF6B6B)
                    .field(EmbedField {
                        name: "Usage".to_string(),
                        value: ".process <list|kill|info> [pid|name]".to_string(),
                        inline: false,
                    })
                    .field(EmbedField {
                        name: "Actions".to_string(),
                        value: "**list** - List all processes\n**kill** - Kill a process by PID\n**info** - Get detailed process info\n**installed** - List all installed applications".to_string(),
                        inline: false,
                    })
                    .footer(EmbedFooterBuilder::new("Kurinium System Commands"))
                    .build();

                http.create_message(msg.channel_id).embeds(&[embed]).await?;
                return Ok(());
            }
        };

        match action {
            "list" => self.list_processes(http, msg).await,
            "installed" => self.list_installed_apps(http, msg).await,
            "kill" => {
                let target = match args.next() {
                    Some(target) => target,
                    None => {
                        http.create_message(msg.channel_id)
                            .content("**Error**: Please provide a PID to kill")
                            .await?;
                        return Ok(());
                    }
                };
                self.kill_process(http, msg, target).await
            }
            "info" => {
                let target = match args.next() {
                    Some(target) => target,
                    None => {
                        http.create_message(msg.channel_id)
                            .content("**Error**: Please provide a PID or process name")
                            .await?;
                        return Ok(());
                    }
                };
                self.process_info(http, msg, target).await
            }
            _ => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "**Error**: Unknown action '{}'. Use: list, kill, info, or installed",
                        action
                    ))
                    .await?;
                Ok(())
            }
        }
    }
}

impl ProcessCommand {
    async fn list_processes(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let mut manager = ProcessManager::new();
        let processes = manager.list_processes();

        if processes.is_empty() {
            http.create_message(msg.channel_id)
                .content("No processes found")
                .await?;
            return Ok(());
        }

        // Limit to first 50 processes to avoid Discord message limits
        let processes: Vec<_> = processes.into_iter().take(50).collect();

        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut count = 0;

        for process in processes {
            count += 1;
            let line = format!(
                "`{:6}` | `{:<20}` | `{:<8}` | `{}`\n",
                process.pid,
                truncate_string(&process.name, 20),
                format_memory_size(process.memory_usage),
                truncate_string(&process.cmdline, 40)
            );

            if current_field.len() + line.len() > 1000 {
                fields.push(EmbedField {
                    name: format!("Processes ({}-{})", fields.len() * 20 + 1, count),
                    value: format!(
                        "```\nPID     | Name                | Memory    | Command\n{}\n```",
                        current_field
                    ),
                    inline: false,
                });
                current_field = line;
            } else {
                current_field.push_str(&line);
            }
        }

        if !current_field.is_empty() {
            fields.push(EmbedField {
                name: format!("Processes ({}-{})", fields.len() * 20 + 1, count),
                value: format!(
                    "```\nPID     | Name                | Memory    | Command\n{}\n```",
                    current_field
                ),
                inline: false,
            });
        }

        let mut embed = EmbedBuilder::new()
            .title("Process List")
            .description(format!(
                "Showing {} processes (first 50)",
                manager.get_system_info().process_count
            ))
            .color(0x4ECDC4);

        for field in fields {
            embed = embed.field(field);
        }

        let embed = embed
            .footer(EmbedFooterBuilder::new("Kurinium Process Manager"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }

    async fn kill_process(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        target: &str,
    ) -> Result<()> {
        // Parse target as PID
        let pid = match target.parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => {
                http.create_message(msg.channel_id)
                    .content("**Error**: Please provide a valid PID (numeric value)")
                    .await?;
                return Ok(());
            }
        };

        let mut manager = ProcessManager::new();

        match manager.kill_process(pid) {
            Ok(_) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Process {} terminated successfully", pid))
                    .await?;
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to kill process {}: {}", pid, e))
                    .await?;
            }
        }

        Ok(())
    }

    async fn process_info(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        target: &str,
    ) -> Result<()> {
        let mut manager = ProcessManager::new();

        let processes = if let Ok(pid) = target.parse::<u32>() {
            if let Some(process) = manager.get_process_by_pid(pid) {
                vec![process]
            } else {
                manager.find_processes_by_name(target)
            }
        } else {
            manager.find_processes_by_name(target)
        };

        if processes.is_empty() {
            http.create_message(msg.channel_id)
                .content(&format!("No processes found matching '{}'", target))
                .await?;
            return Ok(());
        }

        // Show info for up to 5 matching processes
        let processes: Vec<_> = processes.into_iter().take(5).collect();

        for (i, process) in processes.iter().enumerate() {
            let start_time = chrono::DateTime::from_timestamp(process.start_time as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let fields = vec![
                EmbedField {
                    name: "PID".to_string(),
                    value: process.pid.to_string(),
                    inline: true,
                },
                EmbedField {
                    name: "Name".to_string(),
                    value: process.name.clone(),
                    inline: true,
                },
                EmbedField {
                    name: "Status".to_string(),
                    value: process.status.clone(),
                    inline: true,
                },
                EmbedField {
                    name: "Memory Usage".to_string(),
                    value: format_memory_size(process.memory_usage),
                    inline: true,
                },
                EmbedField {
                    name: "CPU Usage".to_string(),
                    value: format_cpu_usage(process.cpu_usage),
                    inline: true,
                },
                EmbedField {
                    name: "Start Time".to_string(),
                    value: start_time,
                    inline: true,
                },
                EmbedField {
                    name: "Parent PID".to_string(),
                    value: process
                        .parent_pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                    inline: true,
                },
                EmbedField {
                    name: "Command Line".to_string(),
                    value: if process.cmdline.is_empty() {
                        "N/A".to_string()
                    } else {
                        truncate_string(&process.cmdline, 500)
                    },
                    inline: false,
                },
            ];

            let title = if processes.len() > 1 {
                format!(
                    "Process Info {}/{} - {}",
                    i + 1,
                    processes.len(),
                    process.name
                )
            } else {
                format!("Process Info - {}", process.name)
            };

            let mut embed = EmbedBuilder::new().title(title).color(0x45B7D1);

            for field in fields {
                embed = embed.field(field);
            }

            let embed = embed
                .footer(EmbedFooterBuilder::new("Kurinium Process Manager"))
                .build();

            http.create_message(msg.channel_id).embeds(&[embed]).await?;
        }

        if processes.len() > 5 {
            http.create_message(msg.channel_id)
                .content(&format!(
                    "Showing 5 of {} matching processes. Use more specific search terms.",
                    processes.len()
                ))
                .await?;
        }

        Ok(())
    }

    async fn list_installed_apps(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        http.create_message(msg.channel_id)
            .content("Gathering installed applications...")
            .await?;

        use winreg::enums::*;
        use winreg::RegKey;

        let mut apps = Vec::new();
        // 64
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(uninstall) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
            for key_name in uninstall.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(key) = uninstall.open_subkey(&key_name) {
                    let display_name: Result<String, _> = key.get_value("DisplayName");
                    let display_version: Result<String, _> = key.get_value("DisplayVersion");
                    if let Ok(name) = display_name {
                        let version = display_version.unwrap_or_else(|_| "Unknown".to_string());
                        apps.push(format!("{} - {}", name, version));
                    }
                }
            }
        }
        // 32
        if let Ok(uninstall) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
            for key_name in uninstall.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(key) = uninstall.open_subkey(&key_name) {
                    let display_name: Result<String, _> = key.get_value("DisplayName");
                    let display_version: Result<String, _> = key.get_value("DisplayVersion");
                    if let Ok(name) = display_name {
                        let version = display_version.unwrap_or_else(|_| "Unknown".to_string());
                        let app_info = format!("{} - {}", name, version);
                        if !apps.contains(&app_info) {
                            apps.push(app_info);
                        }
                    }
                }
            }
        }
        // current user
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(uninstall) = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
            for key_name in uninstall.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(key) = uninstall.open_subkey(&key_name) {
                    let display_name: Result<String, _> = key.get_value("DisplayName");
                    let display_version: Result<String, _> = key.get_value("DisplayVersion");
                    if let Ok(name) = display_name {
                        let version = display_version.unwrap_or_else(|_| "Unknown".to_string());
                        let app_info = format!("{} - {}", name, version);
                        if !apps.contains(&app_info) {
                            apps.push(app_info);
                        }
                    }
                }
            }
        }
        apps.sort();
        let content = format!(
            "Installed Applications ({})\n{}\n\n{}",
            apps.len(),
            "=".repeat(50),
            apps.join("\n")
        );
        let attachment = Attachment::from_bytes(
            "installed_apps.txt".to_string(),
            content.into_bytes(),
            1
        );
        http.create_message(msg.channel_id)
            .content(&format!("Found {} installed applications", apps.len()))
            .attachments(&[attachment])
            .await?;
        

        Ok(())
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
