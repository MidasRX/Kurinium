use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub struct UnrarCommand;

#[async_trait]
impl BotCommand for UnrarCommand {
    fn name(&self) -> &str { "unrar" }
    fn description(&self) -> &str { "Extract RAR archive (requires WinRAR or unrar)" }
    fn category(&self) -> &str { "filesystem" }
    fn usage(&self) -> &str { ".unrar <file.rar> [password]" }
    fn examples(&self) -> &'static [&'static str] { 
        &[".unrar archive.rar", ".unrar protected.rar mypassword"] 
    }
    fn aliases(&self) -> &'static [&'static str] { &["extractrar"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, mut args: Arguments) -> Result<()> {
        let rar_path = args.next().unwrap_or("").to_string();
        let password_owned = args.rest();
        let password = password_owned.trim();

        if rar_path.is_empty() {
            http.create_message(msg.channel_id)
                .content("ERROR: Please provide a RAR file. Usage: `.unrar <file.rar> [password]`")
                .await?;
            return Ok(());
        }

        let path = Path::new(&rar_path);

        if !path.exists() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: File not found: `{}`", rar_path))
                .await?;
            return Ok(());
        }

        if !path.is_file() {
            http.create_message(msg.channel_id)
                .content(&format!("ERROR: Path is not a file: `{}`", rar_path))
                .await?;
            return Ok(());
        }

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !extension.eq_ignore_ascii_case("rar") && !extension.eq_ignore_ascii_case("r00") {
            http.create_message(msg.channel_id)
                .content(&format!("WARNING: File `{}` does not have a .rar extension. This may not be a RAR archive.", rar_path))
                .await?;
        }

        // Get absolute path for destination
        let destination = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()
            .unwrap_or_else(|_| {
                std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
            });

        let thinking_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Testing archive `{}`...", rar_path))
            .await?;
        let thinking_message = thinking_msg.model().await?;

        // First, test the archive
        match Self::test_archive(path, password) {
            Ok(test_msg) => {
                http.update_message(msg.channel_id, thinking_message.id)
                    .content(Some(&format!("Archive OK. Extracting `{}`...", rar_path)))
                    .await?;
                
                if !test_msg.is_empty() {
                    let _ = http.create_message(msg.channel_id)
                        .content(&format!("Archive test result:\n```\n{}\n```", 
                            if test_msg.len() > 500 { &test_msg[..500] } else { &test_msg }
                        ))
                        .await;
                }
            }
            Err(e) => {
                http.update_message(msg.channel_id, thinking_message.id)
                    .content(Some(&format!("WARNING: Archive test failed: {}\n\nAttempting extraction anyway...", e)))
                    .await?;
            }
        }

        match Self::extract_rar(path, &destination, password) {
            Ok((output, tool_used)) => {
                let success_msg = if password.is_empty() {
                    format!(
                        "SUCCESS: **Extracted successfully!**\n**File:** `{}`\n**Destination:** `{}`\n**Tool:** `{}`",
                        rar_path,
                        destination.display(),
                        tool_used
                    )
                } else {
                    format!(
                        "SUCCESS: **Extracted successfully with password!**\n**File:** `{}`\n**Destination:** `{}`\n**Tool:** `{}`",
                        rar_path,
                        destination.display(),
                        tool_used
                    )
                };

                let mut content = success_msg;
                if !output.is_empty() {
                    let truncated = if output.len() > 500 {
                        format!("{}...", &output[..500])
                    } else {
                        output
                    };
                    content.push_str(&format!("\n\n**Output:**\n```\n{}\n```", truncated));
                }

                http.update_message(msg.channel_id, thinking_message.id)
                    .content(Some(&content))
                    .await?;
            }
            Err(e) => {
                let error_msg = if e.to_string().contains("not found") || e.to_string().contains("not recognized") {
                    format!(
                        "ERROR: RAR extraction failed. WinRAR or unrar not found.\n\n**Install WinRAR:**\n- Download from https://www.rarlab.com/download.htm\n- Add to PATH: `C:\\Program Files\\WinRAR\\`\n\n**Error:** {}",
                        e
                    )
                } else if e.to_string().contains("password") || e.to_string().contains("incorrect") {
                    format!("ERROR: Wrong password or archive is corrupted.\n\n**Error:** {}", e)
                } else {
                    format!("ERROR: Failed to extract `{}`: {}", rar_path, e)
                };

                http.update_message(msg.channel_id, thinking_message.id)
                    .content(Some(&error_msg))
                    .await?;
            }
        }

        Ok(())
    }
}

impl UnrarCommand {
    fn interpret_exit_code(code: i32) -> &'static str {
        match code {
            0 => "Success",
            1 => "Warning (non-fatal error)",
            2 => "Fatal error",
            3 => "CRC error",
            4 => "Attempt to modify locked archive",
            5 => "Write error",
            6 => "File open error",
            7 => "Wrong command line option",
            8 => "Not enough memory",
            9 => "File create error or no files to extract",
            10 => "No files to extract",
            11 => "Incorrect password",
            255 => "User stopped the process",
            _ => "Unknown error",
        }
    }

    #[cfg(windows)]
    fn test_archive(rar_path: &Path, password: &str) -> Result<String> {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let unrar_paths = vec![
            ("C:\\Program Files\\WinRAR\\UnRAR.exe", true),
            ("C:\\Program Files\\WinRAR\\WinRAR.exe", true),
            ("C:\\Program Files (x86)\\WinRAR\\UnRAR.exe", true),
            ("C:\\Program Files (x86)\\WinRAR\\WinRAR.exe", true),
            ("unrar", false),
        ];

        for (unrar_exe, check_exists) in unrar_paths {
            if check_exists && !Path::new(unrar_exe).exists() {
                continue;
            }

            let mut cmd = Command::new(unrar_exe);
            cmd.arg("t")
                .arg("-y");

            if !password.is_empty() {
                cmd.arg(format!("-p{}", password));
            } else {
                cmd.arg("-p-");
            }

            cmd.arg(rar_path)
                .creation_flags(CREATE_NO_WINDOW);

            if let Ok(output) = cmd.output() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    return Ok(stdout);
                } else {
                    let exit_code = output.status.code().unwrap_or(-1);
                    let exit_desc = Self::interpret_exit_code(exit_code);
                    return Err(anyhow::anyhow!(
                        "Archive test failed - Exit code: {} ({})\n{}\n{}", 
                        exit_code, exit_desc, stdout, stderr
                    ));
                }
            }
        }

        Err(anyhow::anyhow!("Could not find WinRAR to test archive"))
    }

    #[cfg(not(windows))]
    fn test_archive(rar_path: &Path, password: &str) -> Result<String> {
        let mut cmd = Command::new("unrar");
        cmd.arg("t").arg("-y");

        if !password.is_empty() {
            cmd.arg(format!("-p{}", password));
        } else {
            cmd.arg("-p-");
        }

        cmd.arg(rar_path);

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        
        if output.status.success() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);
            let exit_desc = Self::interpret_exit_code(exit_code);
            Err(anyhow::anyhow!(
                "Archive test failed - Exit code: {} ({})\n{}\n{}", 
                exit_code, exit_desc, stdout, stderr
            ))
        }
    }

    #[cfg(windows)]
    fn extract_rar(rar_path: &Path, destination: &Path, password: &str) -> Result<(String, String)> {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let unrar_paths = vec![
            ("unrar", false),
            ("C:\\Program Files\\WinRAR\\UnRAR.exe", true),
            ("C:\\Program Files\\WinRAR\\WinRAR.exe", true),
            ("C:\\Program Files (x86)\\WinRAR\\UnRAR.exe", true),
            ("C:\\Program Files (x86)\\WinRAR\\WinRAR.exe", true),
        ];

        let mut last_error = None;
        let mut tried_paths = Vec::new();

        for (unrar_exe, check_exists) in unrar_paths {
            if check_exists && !Path::new(unrar_exe).exists() {
                continue;
            }

            tried_paths.push(unrar_exe);

            let mut cmd = Command::new(unrar_exe);
            cmd.arg("x")
                .arg("-y")
                .arg("-o+")
                .arg("-ilog")
                .arg("-idq");

            if !password.is_empty() {
                cmd.arg(format!("-p{}", password));
            } else {
                cmd.arg("-p-");
            }

            // Add trailing backslash to destination on Windows
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
                        let exit_code = output.status.code().unwrap_or(-1);
                        
                        if stderr.contains("password") || stdout.contains("password") 
                            || stderr.contains("encrypted") || stdout.contains("encrypted") {
                            return Err(anyhow::anyhow!("Archive is encrypted. Please provide password using: .unrar <file.rar> <password>"));
                        }
                        
                        if stderr.contains("CRC failed") || stdout.contains("CRC failed") 
                            || stderr.contains("corrupt") || stdout.contains("corrupt") {
                            return Err(anyhow::anyhow!("Archive is corrupted or damaged"));
                        }

                        if stderr.contains("not RAR archive") || stdout.contains("not RAR archive") {
                            return Err(anyhow::anyhow!("File is not a valid RAR archive"));
                        }
                        
                        let exit_code_desc = Self::interpret_exit_code(exit_code);
                        
                        let error_msg = if !stdout.is_empty() || !stderr.is_empty() {
                            format!(
                                "Exit code: {} ({})\nOutput:\n{}\nError:\n{}\n\nTip: If archive seems valid, try:\n- Different extraction tool\n- Check file permissions on destination\n- Ensure archive is fully downloaded",
                                exit_code, exit_code_desc, stdout.trim(), stderr.trim()
                            )
                        } else {
                            format!(
                                "Exit code: {} ({})\n\nPossible causes:\n- Archive file is incomplete or corrupted\n- Unsupported RAR version\n- Insufficient disk space\n- Permission issues in destination folder\n\nTip: Try extracting manually with WinRAR GUI to see detailed error.",
                                exit_code, exit_code_desc
                            )
                        };
                        
                        last_error = Some(anyhow::anyhow!("{}", error_msg));
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to run {}: {}", unrar_exe, e));
                    continue;
                }
            }
        }

        let paths_checked = if tried_paths.is_empty() {
            "No WinRAR installation found".to_string()
        } else {
            format!("Tried paths: {}", tried_paths.join(", "))
        };

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("WinRAR or unrar not found. Please install WinRAR and add it to PATH.\n{}", paths_checked)
        }))
    }

    #[cfg(not(windows))]
    fn extract_rar(rar_path: &Path, destination: &Path, password: &str) -> Result<(String, String)> {
        let mut cmd = Command::new("unrar");
        cmd.arg("x")
            .arg("-y")
            .arg("-o+");

        if !password.is_empty() {
            cmd.arg(format!("-p{}", password));
        } else {
            cmd.arg("-p-");
        }

        cmd.arg(rar_path).arg(destination);

        let output = cmd.output()?;

        if output.status.success() {
            Ok((String::from_utf8_lossy(&output.stdout).to_string(), "unrar".to_string()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let exit_code = output.status.code().unwrap_or(-1);
            Err(anyhow::anyhow!("Extraction failed (exit code: {}): stdout: {} stderr: {}", exit_code, stdout, stderr))
        }
    }
}
