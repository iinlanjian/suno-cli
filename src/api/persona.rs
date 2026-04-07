use super::SunoClient;
use super::types::PersonaInfo;
use crate::errors::CliError;

impl SunoClient {
    /// Fetch voice persona details.
    /// GET /api/persona/get-persona-paginated/{persona_id}/?page=0
    pub async fn get_persona(&self, persona_id: &str) -> Result<PersonaInfo, CliError> {
        let resp = self
            .get(&format!(
                "/api/persona/get-persona-paginated/{persona_id}/?page=0"
            ))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;

        let body: serde_json::Value = resp.json().await?;

        if let Some(first) = body
            .get("items")
            .and_then(|v| v.as_array())
            .and_then(|items| items.first())
        {
            let info: PersonaInfo = serde_json::from_value(first.clone())?;
            return Ok(info);
        }

        let info: PersonaInfo = serde_json::from_value(body)?;
        Ok(info)
    }
}
