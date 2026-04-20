use super::SunoClient;
use crate::errors::CliError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct UploadAudioRequest {
    extension: String,
}

/// Response from POST /api/uploads/audio/ — S3 presigned POST credentials.
#[derive(Debug, Deserialize, Serialize)]
pub struct UploadAudioResponse {
    /// The upload ID assigned by Suno
    pub id: String,
    /// S3 endpoint URL (e.g. https://suno-uploads.s3.amazonaws.com/)
    pub url: String,
    /// S3 presigned POST fields — must be sent as multipart form-data
    pub fields: S3Fields,
    /// Whether the file has been uploaded yet
    pub is_file_uploaded: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct S3Fields {
    #[serde(rename = "Content-Type")]
    pub content_type: String,
    pub key: String,
    #[serde(rename = "AWSAccessKeyId")]
    pub aws_access_key_id: String,
    pub policy: String,
    pub signature: String,
}

/// Determine MIME content type from file extension.
pub fn content_type_for_ext(ext: &str) -> &'static str {
    match ext {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "wma" => "audio/x-ms-wma",
        _ => "application/octet-stream",
    }
}

impl SunoClient {
    /// Step 1: Request an upload slot for an audio file.
    /// POST /api/uploads/audio/ with {"extension": "mp3"}
    /// Returns S3 presigned POST credentials.
    pub async fn upload_audio_init(
        &self,
        extension: &str,
    ) -> Result<UploadAudioResponse, CliError> {
        let body = UploadAudioRequest {
            extension: extension.to_string(),
        };
        let resp = self.post("/api/uploads/audio/").json(&body).send().await?;
        let resp = self.check_response(resp).await?;
        let upload_resp: UploadAudioResponse = resp.json().await.map_err(|e| {
            CliError::Api {
                code: "parse_error",
                message: format!("Failed to parse upload response: {e}"),
            }
        })?;
        Ok(upload_resp)
    }

    /// Step 2: Upload the actual audio bytes to S3 via presigned POST (multipart form-data).
    pub async fn upload_audio_to_s3(
        &self,
        init: &UploadAudioResponse,
        data: Vec<u8>,
    ) -> Result<(), CliError> {
        let file_name = init
            .fields
            .key
            .rsplit('/')
            .next()
            .unwrap_or("audio.mp3");

        let form = reqwest::multipart::Form::new()
            .text("Content-Type", init.fields.content_type.clone())
            .text("key", init.fields.key.clone())
            .text("AWSAccessKeyId", init.fields.aws_access_key_id.clone())
            .text("policy", init.fields.policy.clone())
            .text("signature", init.fields.signature.clone())
            .part(
                "file",
                reqwest::multipart::Part::bytes(data)
                    .file_name(file_name.to_string())
                    .mime_str(&init.fields.content_type)
                    .unwrap_or_else(|_| reqwest::multipart::Part::bytes(Vec::new()).file_name("audio")),
            );

        let resp = self
            .client
            .post(&init.url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| CliError::Api {
                code: "upload_failed",
                message: format!("S3 upload failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                code: "upload_failed",
                message: format!("S3 upload failed (HTTP {status}): {body}"),
            });
        }

        Ok(())
    }
}
