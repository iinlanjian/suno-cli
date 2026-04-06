use super::SunoClient;
use super::types::{Clip, StemsRequest};
use crate::errors::CliError;

impl SunoClient {
    pub async fn stems(&self, clip_id: &str) -> Result<Clip, CliError> {
        let resp = self
            .post("/api/generate/stems/")
            .json(&StemsRequest {
                clip_id: clip_id.to_string(),
            })
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }
}
