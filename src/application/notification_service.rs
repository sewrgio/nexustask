use serde::{Serialize, Deserialize};
use uuid::Uuid;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub url: String,
    pub events: Vec<String>, // e.g., ["task.created", "task.updated", "user.joined"]
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub workspace_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryResult {
    pub webhook_id: Uuid,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub delivered_at: DateTime<Utc>,
}

pub struct NotificationService {
    http_client: Client,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    pub async fn send_webhook(
        &self,
        webhook: &WebhookConfig,
        payload: &NotificationPayload,
    ) -> Result<WebhookDeliveryResult> {
        if !webhook.is_active {
            return Ok(WebhookDeliveryResult {
                webhook_id: webhook.id,
                success: false,
                status_code: None,
                response_body: None,
                error: Some("Webhook is inactive".to_string()),
                delivered_at: Utc::now(),
            });
        }

        // Check if the event type is in the webhook's subscribed events
        if !webhook.events.contains(&payload.event_type) {
            return Ok(WebhookDeliveryResult {
                webhook_id: webhook.id,
                success: false,
                status_code: None,
                response_body: None,
                error: Some("Event not subscribed".to_string()),
                delivered_at: Utc::now(),
            });
        }

        // Prepare the request
        let mut request = self.http_client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "NexusTask/2.0")
            .json(payload);

        // Add signature header if secret is configured
        if let Some(secret) = &webhook.secret {
            let signature = self.generate_signature(payload, secret)?;
            request = request.header("X-NexusTask-Signature", signature);
        }

        // Send the request
        match request.send().await {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let response_body = response.text().await.ok();

                Ok(WebhookDeliveryResult {
                    webhook_id: webhook.id,
                    success: status_code >= 200 && status_code < 300,
                    status_code: Some(status_code),
                    response_body,
                    error: None,
                    delivered_at: Utc::now(),
                })
            }
            Err(e) => {
                Ok(WebhookDeliveryResult {
                    webhook_id: webhook.id,
                    success: false,
                    status_code: None,
                    response_body: None,
                    error: Some(e.to_string()),
                    delivered_at: Utc::now(),
                })
            }
        }
    }

    pub async fn send_to_multiple_webhooks(
        &self,
        webhooks: &[WebhookConfig],
        payload: &NotificationPayload,
    ) -> Result<Vec<WebhookDeliveryResult>> {
        let mut results = Vec::new();

        for webhook in webhooks {
            let result = self.send_webhook(webhook, payload).await?;
            results.push(result);
        }

        Ok(results)
    }

    fn generate_signature(&self, payload: &NotificationPayload, secret: &str) -> Result<String> {
        let payload_json = serde_json::to_string(payload)?;
        
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("Invalid secret key: {}", e))?;
        
        mac.update(payload_json.as_bytes());
        let signature = mac.finalize().into_bytes();
        
        Ok(hex::encode(signature))
    }

    pub fn create_webhook_config(
        &self,
        workspace_id: Uuid,
        url: String,
        events: Vec<String>,
        secret: Option<String>,
    ) -> WebhookConfig {
        WebhookConfig {
            id: Uuid::new_v4(),
            workspace_id,
            url,
            events,
            secret,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    pub fn create_notification_payload(
        &self,
        event_type: String,
        entity_type: String,
        entity_id: String,
        workspace_id: String,
        user_id: Option<String>,
        data: serde_json::Value,
    ) -> NotificationPayload {
        NotificationPayload {
            event_type,
            entity_type,
            entity_id,
            workspace_id,
            user_id,
            timestamp: Utc::now(),
            data,
        }
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
