use crate::{IntegrationError, Result};
use common::Schedule;
use serde_json::json;

pub struct DiscordClient {
    webhook_url: Option<String>,
    client: reqwest::Client,
}

impl DiscordClient {
    pub fn new() -> Self {
        Self {
            webhook_url: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn configure(&mut self, webhook_url: String) {
        self.webhook_url = Some(webhook_url);
    }

    pub fn is_configured(&self) -> bool {
        self.webhook_url.is_some()
    }

    // Placeholder for sending schedule notification to Discord
    pub async fn send_schedule_notification(
        &self,
        schedule: &Schedule,
        message: Option<String>,
    ) -> Result<()> {
        let Some(webhook_url) = &self.webhook_url else {
            return Err(IntegrationError::NotConfigured);
        };

        // TODO: Implement actual Discord webhook integration
        // This would involve:
        // 1. Formatting the schedule into a Discord-friendly message
        // 2. Creating embeds for better visualization
        // 3. Sending the webhook request

        let payload = json!({
            "content": message.unwrap_or_else(|| "Schedule updated".to_string()),
            "embeds": [{
                "title": format!("Schedule for {}/{}", schedule.month, schedule.year),
                "description": "New office schedule has been generated",
                "color": 5814783,
            }]
        });

        // Placeholder for actual HTTP request
        // self.client.post(webhook_url)
        //     .json(&payload)
        //     .send()
        //     .await?;

        Ok(())
    }

    // Helper to format schedule summary
    fn format_schedule_summary(&self, schedule: &Schedule) -> String {
        let mut summary = format!("**Schedule for {}/{}**\n\n", schedule.month, schedule.year);

        for day in common::Weekday::all() {
            let count = schedule.count_for_day(day);
            summary.push_str(&format!("**{}**: {} employees\n", day, count));
        }

        summary
    }
}

impl Default for DiscordClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_client_configuration() {
        let mut client = DiscordClient::new();
        assert!(!client.is_configured());

        client.configure("https://discord.com/api/webhooks/test".to_string());
        assert!(client.is_configured());
    }
}
