use super::SunoClient;
use super::types::{Clip, GenerateRequest};
use crate::errors::CliError;

impl SunoClient {
    /// Create a cover of an existing clip.
    /// Posts to `/api/generate/v2-web/` with `cover_clip_id` set.
    /// If no title is provided, auto-generates one from the clip ID.
    pub async fn cover(
        &self,
        clip_id: &str,
        model_key: &str,
        tags: Option<&str>,
        lyrics: &str,
        title: Option<&str>,
    ) -> Result<Vec<Clip>, CliError> {
        let mut req = GenerateRequest::new(model_key, "cover");
        req.generation_type = "SIMPLE_REMIX".to_string();
        req.title = Some(
            title
                .map(String::from)
                .unwrap_or_else(|| format!("cover_{}", &clip_id[..8])),
        );
        req.tags = tags.map(String::from);
        req.prompt = lyrics.to_string();
        req.cover_clip_id = Some(clip_id.to_string());
        self.generate(&req).await
    }
}
