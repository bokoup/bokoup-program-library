use crate::error::AppError;
use serde::{Deserialize, Serialize};

pub struct Clover {
    pub base_url: url::Url,
    pub client: reqwest::Client,
}

impl Clover {
    pub async fn post_device_notification(
        &self,
        merchant_id: &str,
        notification_data: NotificationData,
    ) -> Result<(), AppError> {
        let clover_app_secret = std::env::var("CLOVER_APP_SECRET").unwrap();

        // https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications
        // https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications

        tracing::debug!(
            "post_device_notification_body {:?}",
            serde_json::to_string(&notification_data).unwrap()
        );

        let request = self
            .client
            .post(
                self.base_url
                    .clone()
                    .join(format!("merchants/{merchant_id}/notifications/").as_str())
                    .map_err(|e| AppError::UrlParseError(e))
                    .unwrap(),
            )
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {clover_app_secret}").as_str(),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&notification_data).unwrap())
            .build()
            .unwrap();

        tracing::debug!("post_device_notification_request {:?}", &request);

        let result = self
            .client
            .execute(request)
            .await
            .map_err(|e| AppError::CloverPostError(e))?;

        tracing::debug!("post_device_notification {:?}", &result);
        if result.status() != reqwest::StatusCode::OK {
            return Err(AppError::StatusNotOK(result.status()));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotificationData {
    pub event: String,
    pub time_to_live: u32,
    pub data: String,
}

impl Default for NotificationData {
    fn default() -> Self {
        Self {
            event: "bokoup".to_string(),
            time_to_live: 300,
            data: "".to_string(),
        }
    }
}
