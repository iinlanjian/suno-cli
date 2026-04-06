use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::CliError;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AuthState {
    pub jwt: Option<String>,
    pub cookie: Option<String>,
    pub session_id: Option<String>,
    pub device_id: Option<String>,
}

impl AuthState {
    pub fn load() -> Result<Self, CliError> {
        let path = Self::path();
        if !path.exists() {
            return Err(CliError::AuthMissing);
        }
        let data = std::fs::read_to_string(&path)?;
        serde_json::from_str(&data).map_err(|e| CliError::Config(format!("corrupt auth file: {e}")))
    }

    pub fn save(&self) -> Result<(), CliError> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, &data)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn jwt(&self) -> Result<&str, CliError> {
        self.jwt.as_deref().ok_or(CliError::AuthMissing)
    }

    pub fn device_id(&self) -> &str {
        self.device_id
            .as_deref()
            .unwrap_or("00000000-0000-0000-0000-000000000000")
    }

    pub fn is_jwt_expired(&self) -> bool {
        let Some(jwt) = &self.jwt else { return true };
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return true;
        }
        // Decode claims (with padding tolerance)
        let claims = parts[1];
        let padded = match claims.len() % 4 {
            2 => format!("{claims}=="),
            3 => format!("{claims}="),
            _ => claims.to_string(),
        };
        let Ok(decoded) = BASE64.decode(&padded) else {
            return true;
        };
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
            return true;
        };
        let Some(exp) = value.get("exp").and_then(|v| v.as_u64()) else {
            return true;
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= exp
    }

    fn path() -> PathBuf {
        directories::ProjectDirs::from("com", "suno-cli", "suno-cli")
            .map(|dirs| dirs.config_dir().join("auth.json"))
            .unwrap_or_else(|| PathBuf::from("~/.config/suno-cli/auth.json"))
    }
}

/// Generate the dynamic browser-token header value.
/// Format: {"token":"<base64({"timestamp":<epoch_ms>})>"}
pub fn browser_token() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let payload = format!(r#"{{"timestamp":{ms}}}"#);
    let encoded = BASE64.encode(payload.as_bytes());
    format!(r#"{{"token":"{encoded}"}}"#)
}
