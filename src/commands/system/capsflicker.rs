use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use winapi::um::winuser::{keybd_event, VK_CAPITAL, VK_NUMLOCK, VK_SCROLL, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP};

static CAPSFLICKER_RUNNING: AtomicBool = AtomicBool::new(false);

pub struct CapsFlickerCommand;

#[async_trait]
impl BotCommand for CapsFlickerCommand {
    fn name(&self) -> &str { "capsflicker" }
    fn description(&self) -> &str { "Spam toggle all lock keys (Caps Lock, Num Lock, Scroll Lock)" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".capsflicker <on|off>" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".capsflicker on",
            ".capsflicker off",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["flicker", "lockspam"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = match args.next() {
            Some(a) => a.to_lowercase(),
            None => {
                http.create_message(msg.channel_id)
                    .content("**Usage**: `.capsflicker <on|off>`")
                    .await?;
                return Ok(());
            }
        };

        let enable = match action.as_str() {
            "on" | "enable" | "start" => true,
            "off" | "disable" | "stop" => false,
            _ => {
                http.create_message(msg.channel_id)
                    .content("**Error**: Action must be `on` or `off`")
                    .await?;
                return Ok(());
            }
        };

        if enable {
            if CAPSFLICKER_RUNNING.load(Ordering::SeqCst) {
                http.create_message(msg.channel_id)
                    .content("**Error**: Already running")
                    .await?;
                return Ok(());
            }
            CAPSFLICKER_RUNNING.store(true, Ordering::SeqCst);
            thread::spawn(move || {
                unsafe {
                    while CAPSFLICKER_RUNNING.load(Ordering::SeqCst) {
                        // Toggle Caps Lock
                        keybd_event(VK_CAPITAL as u8, 0x45, KEYEVENTF_EXTENDEDKEY, 0);
                        keybd_event(VK_CAPITAL as u8, 0x45, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
                        thread::sleep(Duration::from_millis(50));
                        // Toggle Num Lock
                        keybd_event(VK_NUMLOCK as u8, 0x45, KEYEVENTF_EXTENDEDKEY, 0);
                        keybd_event(VK_NUMLOCK as u8, 0x45, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
                        thread::sleep(Duration::from_millis(50));
                        // Toggle Scroll Lock
                        keybd_event(VK_SCROLL as u8, 0x46, KEYEVENTF_EXTENDEDKEY, 0);
                        keybd_event(VK_SCROLL as u8, 0x46, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            });
            http.create_message(msg.channel_id)
                .content("**Started spamming all lock keys**")
                .await?;
        } else {
            if !CAPSFLICKER_RUNNING.load(Ordering::SeqCst) {
                http.create_message(msg.channel_id)
                    .content("**Error**: CapsFlicker is not running")
                    .await?;
                return Ok(());
            }
            CAPSFLICKER_RUNNING.store(false, Ordering::SeqCst);
            http.create_message(msg.channel_id)
                .content("**Stopped flickering**")
                .await?;
        }

        Ok(())
    }
}
