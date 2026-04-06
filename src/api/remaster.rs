use super::SunoClient;
use super::types::{Clip, RemasterRequest};
use crate::errors::CliError;

impl SunoClient {
    pub async fn remaster(&self, clip_id: &str, model_key: &str) -> Result<Clip, CliError> {
        let resp = self
            .post("/api/remaster/")
            .json(&RemasterRequest {
                clip_id: clip_id.to_string(),
                remaster_model: model_key.to_string(),
            })
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }
}
