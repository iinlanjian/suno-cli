use super::SunoClient;
use super::types::{Clip, GenerateRequest};
use crate::errors::CliError;

impl SunoClient {
    /// Create a cover of an existing clip.
    /// Routes through /api/generate/v2/ with task="cover" + cover_clip_id.
    pub async fn cover(
        &self,
        clip_id: &str,
        model_key: &str,
        tags: Option<&str>,
    ) -> Result<Vec<Clip>, CliError> {
        let req = GenerateRequest {
            mv: model_key.to_string(),
            prompt: None,
            gpt_description_prompt: None,
            title: None,
            tags: tags.map(String::from),
            negative_tags: None,
            make_instrumental: false,
            generation_type: None,
            token: None,
            continue_clip_id: None,
            continue_at: None,
            task: Some("cover".into()),
            persona_id: None,
            cover_clip_id: Some(clip_id.to_string()),
            metadata: None,
        };
        self.generate(&req).await
    }
}
