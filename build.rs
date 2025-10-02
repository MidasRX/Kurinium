use std::fs;
use std::path::Path;
use winres::WindowsResource;
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config_path = Path::new("src/config.rs");
    let config_content = fs::read_to_string(config_path).context("Failed to read src/config.rs")?;

    let get_val = |key| {
        config_content
            .lines()
            .find(|line| line.trim().starts_with(&format!("{}:", key)))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|val| val.trim().trim_matches(',').trim_matches('"').split('/').next())
            .unwrap_or("")
            .to_string()
    };

    let file_ver_str = get_val("file_version");
    let file_name_str = get_val("file_name");
    let product_name = get_val("product_name");
    let description = get_val("description");
    let company_name = get_val("company_name");

    if cfg!(target_os = "windows") {
        let ver_parts: Vec<u64> = file_ver_str
            .split('.')
            .map(|s| s.parse::<u64>().unwrap_or(0))
            .collect();

        let file_ver_u64 = if ver_parts.len() == 4 {
            (ver_parts[0] << 48) | (ver_parts[1] << 32) | (ver_parts[2] << 16) | ver_parts[3]
        } else {
            0x0001000000000000
        };

        WindowsResource::new()
            .set_icon("assets/kurinium.ico")
            .set_version_info(winres::VersionInfo::FILEVERSION, file_ver_u64)
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, file_ver_u64)
            .set("FileVersion", &file_ver_str)
            .set("ProductVersion", &file_ver_str)
            .set("ProductName", &product_name)
            .set("FileDescription", &description)
            .set("CompanyName", &company_name)
            .set("InternalName", &file_name_str) 
            .set("OriginalFilename", &file_name_str)
            .compile()?;
    }
    
    println!("cargo:rerun-if-changed=src/config.rs");

    Ok(())
}