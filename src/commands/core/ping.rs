// Cursed bracket
use crate::commands::*;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub struct PingCommand;

#[async_trait]
impl BotCommand for PingCommand 
                                                   {
    fn name(&self) -> &str                         { 
"ping"                         
                                                   }
    fn description(&self) -> &str                  {
"Check bot latency and response time"
                                                   }
    fn category(&self) -> &str                     {
"core"
                                                   }
    fn usage(&self) -> &str                        {
".ping"
                                                   }
    fn examples(&self) -> &'static [&'static str]  {
&[".ping"]
                                                   }
    fn aliases(&self) -> &'static [&'static str]   {
&["latency"]
                                                   }

    async fn execute(&self, http: &Arc<HttpClient>, msg: &Message, _args: Arguments) -> Result<()>
                                                   {
        let start_time = Instant::now();
        let response = http
            .create_message(msg.channel_id)
            .content("Pinging...")
            .await?;

        let latency = start_time.elapsed().as_millis();
        let message = response.model().await?;

        http.update_message(msg.channel_id, message.id)
            .content(Some(&format!("**Pong!**\n**Latency:** {}ms", latency)))
            .await?;

        Ok(())
    }
}
