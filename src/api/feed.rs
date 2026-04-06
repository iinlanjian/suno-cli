use serde_json::json;

use super::SunoClient;
use super::types::FeedResponse;
use crate::errors::CliError;

impl SunoClient {
    pub async fn feed(&self, page: u32) -> Result<FeedResponse, CliError> {
        let resp = self
            .post("/api/feed/v3")
            .json(&json!({ "page": page }))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }
}
