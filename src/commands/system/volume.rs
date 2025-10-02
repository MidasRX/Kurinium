use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

use windows_volume_control::AudioController;

pub struct VolumeCommand;

#[async_trait]
impl BotCommand for VolumeCommand {
    fn name(&self) -> &str { "volume" }
    fn description(&self) -> &str { "Control system volume" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".volume <get|set|up|down|mute|unmute> [value]" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".volume get",
            ".volume set 50",
            ".volume up 10",
            ".volume down 10",
            ".volume mute",
            ".volume unmute",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["vol"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = match args.next() {
            Some(action) => action,
            None => {
                http.create_message(msg.channel_id)
                    .content("**Usage**: `.volume <get|set|up|down|mute|unmute> [value]`")
                    .await?;
                return Ok(());
            }
        };

        match action {
            "get" => self.get_volume(http, msg).await,
            "set" => {
                let value = match args.next() {
                    Some(v) => v.parse::<f32>().unwrap_or(50.0),
                    None => {
                        http.create_message(msg.channel_id)
                            .content("**Error**: Please provide a volume level (0-100)")
                            .await?;
                        return Ok(());
                    }
                };
                self.set_volume(http, msg, value).await
            }
            "up" => {
                let amount = args.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(10.0);
                self.change_volume(http, msg, amount, true).await
            }
            "down" => {
                let amount = args.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(10.0);
                self.change_volume(http, msg, amount, false).await
            }
            "mute" => self.mute_volume(http, msg, true).await,
            "unmute" => self.mute_volume(http, msg, false).await,
            _ => {
                http.create_message(msg.channel_id)
                    .content(&format!("**Error**: Unknown action '{}'. Use get, set, up, down, mute, or unmute", action))
                    .await?;
                Ok(())
            }
        }
    }
}

impl VolumeCommand {
    async fn get_volume(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let (volume, is_muted, success) = unsafe {
            let mut controller = AudioController::init(None);
            controller.GetSessions();
            controller.GetDefaultAudioEnpointVolumeControl();
            if let Some(session) = controller.get_session_by_name("master".to_string()) {
                let vol = session.getVolume() * 100.0;
                let muted = session.getMute();
                (vol, muted, true)
            } else {
                (0.0, false, false)
            }
        };
        if success {
            let status = if is_muted { "Muted" } else { "Unmuted" };
            http.create_message(msg.channel_id)
                .content(&format!("**Current Volume**: {:.0}%\n**Status**: {}", volume, status))
                .await?;
        } else {
            http.create_message(msg.channel_id)
                .content("**Error**: Failed to get master audio session")
                .await?;
        }
        Ok(())
    }

    async fn set_volume(&self, http: &Arc<HttpClient>, msg: &Message, level: f32) -> Result<()> {
        if level > 100.0 || level < 0.0 {
            http.create_message(msg.channel_id)
                .content("**Error**: Volume must be between 0 and 100")
                .await?;
            return Ok(());
        }

        let success = unsafe {
            let mut controller = AudioController::init(None);
            controller.GetSessions();
            controller.GetDefaultAudioEnpointVolumeControl();

            if let Some(session) = controller.get_session_by_name("master".to_string()) {
                session.setVolume(level / 100.0);
                true
            } else {
                false
            }
        };

        if success {
            http.create_message(msg.channel_id)
                .content(&format!("**Volume set to**: {:.0}%", level))
                .await?;
        } else {
            http.create_message(msg.channel_id)
                .content("**Error**: Failed to get master audio session")
                .await?;
        }
        
        Ok(())
    }

    async fn change_volume(&self, http: &Arc<HttpClient>, msg: &Message, amount: f32, increase: bool) -> Result<()> {
        let (new_volume, success) = unsafe {
            let mut controller = AudioController::init(None);
            controller.GetSessions();
            controller.GetDefaultAudioEnpointVolumeControl();

            if let Some(session) = controller.get_session_by_name("master".to_string()) {
                let current_volume = session.getVolume() * 100.0;
                let new_vol = if increase {
                    (current_volume + amount).min(100.0)
                } else {
                    (current_volume - amount).max(0.0)
                };

                session.setVolume(new_vol / 100.0);
                (new_vol, true)
            } else {
                (0.0, false)
            }
        };

        if success {
            let action = if increase { "increased" } else { "decreased" };
            http.create_message(msg.channel_id)
                .content(&format!("**Volume {}**: {:.0}%", action, new_volume))
                .await?;
        } else {
            http.create_message(msg.channel_id)
                .content("**Error**: Failed to get master audio session")
                .await?;
        }

        Ok(())
    }

    async fn mute_volume(&self, http: &Arc<HttpClient>, msg: &Message, mute: bool) -> Result<()> {
        let success = unsafe {
            let mut controller = AudioController::init(None);
            controller.GetSessions();
            controller.GetDefaultAudioEnpointVolumeControl();

            if let Some(session) = controller.get_session_by_name("master".to_string()) {
                session.setMute(mute);
                true
            } else {
                false
            }
        };

        if success {
            let status = if mute { "muted" } else { "unmuted" };
            http.create_message(msg.channel_id)
                .content(&format!("**Volume {}**", status))
                .await?;
        } else {
            http.create_message(msg.channel_id)
                .content("**Error**: Failed to get master audio session")
                .await?;
        }

        Ok(())
    }
}
