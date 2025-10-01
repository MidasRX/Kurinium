use crate::commands::*;
use crate::core::screenshot::Screenshot;
use anyhow::Result;
use async_trait::async_trait;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_model::http::attachment::Attachment;

pub struct ScreenshotCommand;

#[async_trait]
impl BotCommand for ScreenshotCommand {
    fn name(&self) -> &str { "screenshot" }
    fn description(&self) -> &str { "Capture the screen and send it as an image" }
    fn category(&self) -> &str { "utility" }
    fn usage(&self) -> &str { ".screenshot" }
    fn examples(&self) -> &'static [&'static str] { &[".screenshot"] }
    fn aliases(&self) -> &'static [&'static str] { &["ss", "capture"] }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()> {
        let thinking_msg = http
            .create_message(msg.channel_id)
            .content("`Capturing screen...`")
            .await?
            .model()
            .await?;

        match Screenshot::capture_as_bytes() {
            Ok((bytes, filename)) => {
                let attachment = Attachment::from_bytes(filename.clone(), bytes, 1);
                http.create_message(msg.channel_id)
                    .content(&format!("**Screenshot captured:** `{}`", filename))
                    .attachments(&[attachment])
                    .await?;
                http.delete_message(thinking_msg.channel_id, thinking_msg.id).await?;
            }
            Err(e) => {
                http.update_message(thinking_msg.channel_id, thinking_msg.id)
                    .content(Some(&format!("**Error**: Failed to capture screenshot: {}", e)))
                    .await?;
            }
        }

        Ok(())
    }
}