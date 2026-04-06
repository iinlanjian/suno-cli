use serde_json::json;

use super::SunoClient;
use crate::errors::CliError;

impl SunoClient {
    pub async fn delete_clips(&self, ids: &[String]) -> Result<(), CliError> {
        let resp = self
            .post("/api/feed/trash")
            .json(&json!({ "ids": ids }))
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }
}
