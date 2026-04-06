use super::SunoClient;
use super::types::{Clip, GenerateRequest};
use crate::errors::CliError;

impl SunoClient {
    /// Remaster a clip with a different model version.
    /// Routes through /api/generate/v2/ with the remaster model key
    /// and cover_clip_id pointing to the original.
    pub async fn remaster(
        &self,
        clip_id: &str,
        remaster_model_key: &str,
    ) -> Result<Vec<Clip>, CliError> {
        let req = GenerateRequest {
            mv: remaster_model_key.to_string(),
            prompt: None,
            gpt_description_prompt: None,
            title: None,
            tags: None,
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
