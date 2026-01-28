//! Webhook notification for critical alerts.
//!
//! Sends alert payloads to an external webhook URL (e.g. Slack incoming
//! webhooks) when critical or warning alerts are created. Notification is
//! fire-and-forget: failures are logged but never block alert creation.

use crate::alerts::Alert;
use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

/// Sends alert notifications to an optional webhook URL.
#[derive(Debug, Clone)]
pub struct WebhookNotifier {
    url: Option<String>,
    client: Client,
}

impl WebhookNotifier {
    /// Create a new notifier. If `url` is `None`, all calls to `notify` are
    /// no-ops.
    pub fn new(url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { url, client }
    }

    /// Asynchronously POST the alert to the webhook. Spawns a background task
    /// so the caller is never blocked.
    pub fn notify(&self, alert: &Alert) {
        // Skip info-level alerts
        if alert.severity != "critical" && alert.severity != "warning" {
            return;
        }

        let url = match &self.url {
            Some(u) => u.clone(),
            None => return,
        };

        let severity_upper = alert.severity.to_uppercase();
        let emoji = if alert.severity == "critical" {
            "\u{1f6a8}" // rotating light
        } else {
            "\u{26a0}\u{fe0f}" // warning sign
        };

        let payload = json!({
            "text": format!("{} *{}* \u{2014} {}", emoji, severity_upper, alert.message),
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!(
                            "{} *{}* alert from *{}*\n{}",
                            emoji, alert.severity, alert.source, alert.message
                        )
                    }
                }
            ]
        });

        let client = self.client.clone();
        tokio::spawn(async move {
            match client.post(&url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    info!(url = %url, "Webhook notification sent");
                }
                Ok(resp) => {
                    error!(
                        url = %url,
                        status = %resp.status(),
                        "Webhook notification failed with non-success status"
                    );
                }
                Err(e) => {
                    error!(url = %url, error = %e, "Webhook notification request failed");
                }
            }
        });
    }
}
