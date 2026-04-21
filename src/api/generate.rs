use super::SunoClient;
use super::types::{Clip, GenerateRequest, GenerateResponse};
use crate::errors::CliError;

impl SunoClient {
    /// Submit a music generation request (custom mode or inspiration mode).
    /// Posts to `/api/generate/v2-web/` — the legacy `/api/generate/v2/`
    /// returns `Token validation failed` since Suno migrated creates to
    /// `v2-web` server-side (verified 2026-04-07).
    /// Wrapped in `with_auth_retry` so a single stale-JWT failure recovers
    /// transparently via Clerk refresh.
    pub async fn generate(&self, req: &GenerateRequest) -> Result<Vec<Clip>, CliError> {
        self.with_auth_retry(|| async {
            let resp = self.post("/api/generate/v2-web/").json(req).send().await?;
            let resp = self.check_response(resp).await?;
            let result: GenerateResponse = resp.json().await?;
            Ok(result.clips)
        })
        .await
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
        let mut delay = std::time::Duration::from_secs(3);

        loop {
            let clips = self.get_clips(ids).await?;
            let all_done = clips
                .iter()
                .all(|c| matches!(c.status.as_str(), "complete" | "error"));

            if all_done {
                // If ALL clips ended in "error", surface it as a generation failure
                // so the user gets a clear non-zero exit instead of silent "success".
                let all_errored = clips
                    .iter()
                    .all(|c| c.status == "error");
                if all_errored {
                    let details: Vec<String> = clips
                        .iter()
                        .map(|c| format!("{} ({})", c.title, c.id))
                        .collect();
                    return Err(CliError::GenerationFailed(format!(
                        "all clips failed — likely content policy or copyright restriction: {}",
                        details.join(", ")
                    )));
                }
                // Partial failure: some succeeded, some errored — warn the user.
                let errored: Vec<&Clip> = clips
                    .iter()
                    .filter(|c| c.status == "error")
                    .collect();
                if !errored.is_empty() {
                    let details: Vec<String> = errored
                        .iter()
                        .map(|c| format!("{} ({})", c.title, c.id))
                        .collect();
                    eprintln!(
                        "Warning: {} clip(s) failed (likely content policy or copyright restriction): {}",
                        errored.len(),
                        details.join(", ")
                    );
                }
                return Ok(clips);
            }
            if start.elapsed() >= timeout {
                return Err(CliError::GenerationFailed(format!(
                    "generation timed out after {timeout_secs}s for {}",
                    ids.join(", ")
                )));
            }
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(std::time::Duration::from_secs(15));
        }
    }

    /// Fetch clips by IDs. Batches in pairs to avoid Suno's limit
    /// (SunoAI-API #49: 4+ IDs from different batches only returns first 2).
    /// Each chunk is wrapped in `with_auth_retry` so long polling waits
    /// survive Suno's JWT staleness window mid-generation.
    pub async fn get_clips(&self, ids: &[String]) -> Result<Vec<Clip>, CliError> {
        let mut all_clips = Vec::new();
        for chunk in ids.chunks(2) {
            let ids_param = chunk.join(",");
            let path = format!("/api/feed/?ids={ids_param}");
            let clips: Vec<Clip> = self
                .with_auth_retry(|| async {
                    let resp = self.get(&path).send().await?;
                    let resp = self.check_response(resp).await?;
                    let clips: Vec<Clip> = resp.json().await?;
                    Ok(clips)
                })
                .await?;
            all_clips.extend(clips);
        }
        Ok(all_clips)
    }
}
