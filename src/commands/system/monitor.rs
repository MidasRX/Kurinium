use crate::commands::Arguments;
use crate::commands::BotCommand;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use sysinfo::{CpuExt, DiskExt, PidExt, ProcessExt, System, SystemExt};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct MonitorCommand;

#[async_trait]
impl BotCommand for MonitorCommand {
    fn name(&self) -> &str { "monitor" }
    fn description(&self) -> &str { "Monitor system resources (cpu, memory, disk, processes)" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".monitor <cpu|memory|disk|processes>" }
    fn examples(&self) -> &'static [&'static str] { &[".monitor cpu", ".monitor memory", ".monitor disk", ".monitor processes"] }
    fn aliases(&self) -> &'static [&'static str] { &["mon", "sysmon"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let resource = match args.next() {
            Some(resource) => resource,
            None => {
                // Show all resources if none specified
                return self.show_all_resources(http, msg).await;
            }
        };

        match resource {
            "cpu" => self.monitor_cpu(http, msg).await,
            "memory" | "mem" => self.monitor_memory(http, msg).await,
            "disk" => self.monitor_disk(http, msg).await,
            "processes" | "procs" => self.monitor_processes(http, msg).await,
            _ => {
                let embed = EmbedBuilder::new()
                    .title("System Monitor")
                    .description("Monitor system resources")
                    .color(0xFF6B6B)
                    .field(EmbedField {
                        name: "Usage".to_string(),
                        value: ".monitor <cpu|memory|disk|processes>".to_string(),
                        inline: false,
                    })
                    .field(EmbedField {
                        name: "Resources".to_string(),
                        value: "**cpu** - CPU usage and information\n**memory** - Memory usage and statistics\n**disk** - Disk usage and information\n**processes** - Process count and top processes".to_string(),
                        inline: false,
                    })
                    .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
                    .build();

                http.create_message(msg.channel_id).embeds(&[embed]).await?;
                Ok(())
            }
        }
    }
}

impl MonitorCommand {
    async fn show_all_resources(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let mut system = System::new_all();
        system.refresh_all();

        // CPU Info
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let cpu_fields = vec![
            EmbedField {
                name: "CPU Usage".to_string(),
                value: format!("{:.1}%", cpu_usage),
                inline: true,
            },
            EmbedField {
                name: "CPU Cores".to_string(),
                value: system.physical_core_count().unwrap_or(1).to_string(),
                inline: true,
            },
        ];

        let mut cpu_embed = EmbedBuilder::new().title("🖥️ CPU Monitor").color(0xFFA500);

        for field in cpu_fields {
            cpu_embed = cpu_embed.field(field);
        }

        let cpu_embed = cpu_embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id)
            .embeds(&[cpu_embed])
            .await?;

        // Memory Info
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = total_memory - used_memory;
        let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

        let memory_fields = vec![
            EmbedField {
                name: "Total Memory".to_string(),
                value: self.format_bytes(total_memory),
                inline: true,
            },
            EmbedField {
                name: "Used Memory".to_string(),
                value: format!(
                    "{} ({:.1}%)",
                    self.format_bytes(used_memory),
                    memory_usage_percent
                ),
                inline: true,
            },
            EmbedField {
                name: "Available Memory".to_string(),
                value: self.format_bytes(available_memory),
                inline: true,
            },
            EmbedField {
                name: "Total Swap".to_string(),
                value: self.format_bytes(system.total_swap()),
                inline: true,
            },
            EmbedField {
                name: "Used Swap".to_string(),
                value: self.format_bytes(system.used_swap()),
                inline: true,
            },
        ];

        let mut memory_embed = EmbedBuilder::new()
            .title("Memory Monitor")
            .color(0x4169E1);

        for field in memory_fields {
            memory_embed = memory_embed.field(field);
        }

        let memory_embed = memory_embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id)
            .embeds(&[memory_embed])
            .await?;

        // Disk Info
        let disks: Vec<_> = system.disks().iter().collect();
        let mut disk_fields = Vec::new();

        for (i, disk) in disks.iter().take(5).enumerate() {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            let mount_point = disk.mount_point().to_string_lossy();
            let disk_name = disk.name().to_string_lossy();

            disk_fields.push(EmbedField {
                name: format!(
                    "Disk {} ({})",
                    i + 1,
                    if disk_name.is_empty() {
                        "Unnamed"
                    } else {
                        &disk_name
                    }
                ),
                value: format!(
                    "Mount: {}\nSize: {}\nUsed: {} ({:.1}%)\nAvailable: {}",
                    mount_point,
                    self.format_bytes(total_space),
                    self.format_bytes(used_space),
                    usage_percent,
                    self.format_bytes(available_space)
                ),
                inline: false,
            });
        }

        let mut disk_embed = EmbedBuilder::new().title("Disk Monitor").color(0x32CD32);

        for field in disk_fields {
            disk_embed = disk_embed.field(field);
        }

        let disk_embed = disk_embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id)
            .embeds(&[disk_embed])
            .await?;

        // Processes Info
        let process_count = system.processes().len();
        let mut top_processes = Vec::new();

        for (pid, process) in system.processes() {
            top_processes.push((
                pid.as_u32(),
                process.name().to_string(),
                process.memory(),
                process.cpu_usage(),
            ));
        }

        // Sort by memory usage
        top_processes.sort_by(|a, b| b.2.cmp(&a.2));
        let top_processes: Vec<_> = top_processes.into_iter().take(5).collect();

        let mut process_fields = Vec::new();
        process_fields.push(EmbedField {
            name: "Total Processes".to_string(),
            value: process_count.to_string(),
            inline: true,
        });

        let mut top_process_list = String::new();
        for (i, (pid, name, memory, cpu)) in top_processes.iter().enumerate() {
            top_process_list.push_str(&format!(
                "{}. {} (PID: {}) - {} - {:.1}% CPU\n",
                i + 1,
                name,
                pid,
                self.format_bytes(*memory),
                cpu
            ));
        }

        process_fields.push(EmbedField {
            name: "Top 5 Processes by Memory".to_string(),
            value: top_process_list,
            inline: false,
        });

        let mut process_embed = EmbedBuilder::new()
            .title("Process Monitor")
            .color(0x9370DB);

        for field in process_fields {
            process_embed = process_embed.field(field);
        }

        let process_embed = process_embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id)
            .embeds(&[process_embed])
            .await?;

        Ok(())
    }

    async fn monitor_cpu(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage();

        let mut fields = Vec::new();
        fields.push(EmbedField {
            name: "Overall CPU Usage".to_string(),
            value: format!("{:.1}%", cpu_usage),
            inline: true,
        });

        fields.push(EmbedField {
            name: "Physical Cores".to_string(),
            value: system.physical_core_count().unwrap_or(1).to_string(),
            inline: true,
        });

        let mut embed = EmbedBuilder::new()
            .title("🖥️ CPU Monitor")
            .description("Current CPU usage and information")
            .color(0xFFA500);

        for field in fields {
            embed = embed.field(field);
        }

        let embed = embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }

    async fn monitor_memory(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let mut system = System::new_all();
        system.refresh_all();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = total_memory - used_memory;
        let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

        let total_swap = system.total_swap();
        let used_swap = system.used_swap();
        let available_swap = total_swap - used_swap;
        let swap_usage_percent = if total_swap > 0 {
            (used_swap as f64 / total_swap as f64) * 100.0
        } else {
            0.0
        };

        let fields = vec![
            EmbedField {
                name: "Total Physical Memory".to_string(),
                value: self.format_bytes(total_memory),
                inline: true,
            },
            EmbedField {
                name: "Used Physical Memory".to_string(),
                value: format!(
                    "{} ({:.1}%)",
                    self.format_bytes(used_memory),
                    memory_usage_percent
                ),
                inline: true,
            },
            EmbedField {
                name: "Available Physical Memory".to_string(),
                value: self.format_bytes(available_memory),
                inline: true,
            },
            EmbedField {
                name: "Total Swap Space".to_string(),
                value: self.format_bytes(total_swap),
                inline: true,
            },
            EmbedField {
                name: "Used Swap Space".to_string(),
                value: if total_swap > 0 {
                    format!(
                        "{} ({:.1}%)",
                        self.format_bytes(used_swap),
                        swap_usage_percent
                    )
                } else {
                    "No swap".to_string()
                },
                inline: true,
            },
            EmbedField {
                name: "Available Swap Space".to_string(),
                value: if total_swap > 0 {
                    self.format_bytes(available_swap)
                } else {
                    "N/A".to_string()
                },
                inline: true,
            },
        ];

        let mut embed = EmbedBuilder::new()
            .title("Memory Monitor")
            .description("Current memory usage and statistics")
            .color(0x4169E1);

        for field in fields {
            embed = embed.field(field);
        }

        let embed = embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }

    async fn monitor_disk(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let mut system = System::new_all();
        system.refresh_all();

        let disks: Vec<_> = system.disks().iter().collect();

        if disks.is_empty() {
            http.create_message(msg.channel_id)
                .content("No disk information available")
                .await?;
            return Ok(());
        }

        let mut fields = Vec::new();

        for (i, disk) in disks.iter().take(5).enumerate() {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            let mount_point = disk.mount_point().to_string_lossy();
            let disk_name = disk.name().to_string_lossy();
            let is_removable = disk.is_removable();

            fields.push(EmbedField {
                name: format!("Disk {} ({})", i + 1, if disk_name.is_empty() { "Unnamed" } else { &disk_name }),
                value: format!(
                    "**Mount Point**: {}\n{}**Total Size**: {}\n**Used Space**: {} ({:.1}%)\n**Available**: {}",
                    mount_point,
                    if is_removable { "**Type**: Removable\n" } else { "" },
                    self.format_bytes(total_space),
                    self.format_bytes(used_space),
                    usage_percent,
                    self.format_bytes(available_space)
                ),
                inline: false,
            });
        }

        if disks.len() > 5 {
            fields.push(EmbedField {
                name: "Note".to_string(),
                value: format!("Showing 5 of {} total disks", disks.len()),
                inline: false,
            });
        }

        let mut embed = EmbedBuilder::new()
            .title("Disk Monitor")
            .description("Disk usage and information")
            .color(0x32CD32);

        for field in fields {
            embed = embed.field(field);
        }

        let embed = embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }

    async fn monitor_processes(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let mut system = System::new_all();
        system.refresh_all();

        let process_count = system.processes().len();

        // Collect processes with their stats
        let mut processes = Vec::new();
        for (pid, process) in system.processes() {
            processes.push((
                pid.as_u32(),
                process.name().to_string(),
                process.memory(),
                process.cpu_usage(),
            ));
        }

        // Sort by memory usage for top processes
        processes.sort_by(|a, b| b.2.cmp(&a.2));

        let mut fields = Vec::new();

        // Total process count
        fields.push(EmbedField {
            name: "Total Running Processes".to_string(),
            value: process_count.to_string(),
            inline: true,
        });

        // Top processes by memory
        let top_memory: Vec<_> = processes.iter().take(5).collect();
        let mut memory_list = String::new();
        for (i, (pid, name, memory, _cpu)) in top_memory.iter().enumerate() {
            memory_list.push_str(&format!(
                "{}. **{}** (PID: {}) - {}\n",
                i + 1,
                name,
                pid,
                self.format_bytes(*memory)
            ));
        }

        fields.push(EmbedField {
            name: "Top 5 Processes by Memory Usage".to_string(),
            value: memory_list,
            inline: false,
        });

        // Top processes by CPU
        processes.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        let top_cpu: Vec<_> = processes.iter().take(5).collect();
        let mut cpu_list = String::new();
        for (i, (pid, name, _memory, cpu)) in top_cpu.iter().enumerate() {
            cpu_list.push_str(&format!(
                "{}. **{}** (PID: {}) - {:.1}% CPU\n",
                i + 1,
                name,
                pid,
                cpu
            ));
        }

        fields.push(EmbedField {
            name: "Top 5 Processes by CPU Usage".to_string(),
            value: cpu_list,
            inline: false,
        });

        let mut embed = EmbedBuilder::new()
            .title("Process Monitor")
            .description("Process information and top resource consumers")
            .color(0x9370DB);

        for field in fields {
            embed = embed.field(field);
        }

        let embed = embed
            .footer(EmbedFooterBuilder::new("Kurinium System Monitor"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;

        Ok(())
    }

    fn format_bytes(&self, bytes: u64) -> String {
        crate::utils::formatting::format_memory_size(bytes)
    }
}
