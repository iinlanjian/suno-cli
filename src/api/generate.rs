use super::SunoClient;
use super::types::{Clip, GenerateRequest, GenerateResponse};
use crate::errors::CliError;

impl SunoClient {
    /// Submit a music generation request (custom mode or inspiration mode).
    pub async fn generate(&self, req: &GenerateRequest) -> Result<Vec<Clip>, CliError> {
        let resp = self.post("/api/generate/v2/").json(req).send().await?;
        let resp = self.check_response(resp).await?;
        let result: GenerateResponse = resp.json().await?;
        Ok(result.clips)
    }

    /// Poll clip status by IDs until all are complete or errored.
    /// "streaming" means still generating — we wait for "complete".
    pub async fn poll_clips(
        &self,
        ids: &[String],
        timeout_secs: u64,
    ) -> Result<Vec<Clip>, CliError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            let clips = self.get_clips(ids).await?;
            let all_done = clips
                .iter()
                .all(|c| c.status == "complete" || c.status == "error");

            if all_done || start.elapsed() > timeout {
                return Ok(clips);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    /// Fetch clips by IDs. Batches in pairs to avoid Suno's limit
    /// (SunoAI-API #49: 4+ IDs from different batches only returns first 2).
    pub async fn get_clips(&self, ids: &[String]) -> Result<Vec<Clip>, CliError> {
        let mut all_clips = Vec::new();
        for chunk in ids.chunks(2) {
            let ids_param = chunk.join(",");
            let resp = self
                .get(&format!("/api/feed/?ids={ids_param}"))
                .send()
                .await?;
            let resp = self.check_response(resp).await?;
            let clips: Vec<Clip> = resp.json().await?;
            all_clips.extend(clips);
        }
        Ok(all_clips)
    }
}
