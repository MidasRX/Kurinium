use crate::config::{KeepActiveConfig};
use std::thread;
use std::time::Duration;
use winapi::um::winuser::{keybd_event, KEYEVENTF_KEYUP, VK_SCROLL};

pub fn start_keep_active(config: &KeepActiveConfig) {
    if !config.enabled {
        return;
    }

    let interval = Duration::from_secs(config.interval_seconds);

    thread::spawn(move || {
        loop {
            unsafe {
                // press scrl lock
                keybd_event(VK_SCROLL as u8, 0, 0, 0);
                // relese scrl lock
                keybd_event(VK_SCROLL as u8, 0, KEYEVENTF_KEYUP, 0);
            }
            thread::sleep(interval);
        }
    });
}