use super::SunoClient;
use super::types::{Clip, CoverRequest};
use crate::errors::CliError;

impl SunoClient {
    pub async fn cover(&self, clip_id: &str, tags: Option<&str>) -> Result<Clip, CliError> {
        let resp = self
            .post("/api/cover/")
            .json(&CoverRequest {
                clip_id: clip_id.to_string(),
                tags: tags.map(String::from),
            })
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }
}
