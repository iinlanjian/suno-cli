use serde::{Deserialize, Serialize};

// --- Billing / Account ---

#[derive(Debug, Deserialize, Serialize)]
pub struct BillingInfo {
    pub credits: u64,
    pub total_credits_left: u64,
    pub monthly_usage: u64,
    pub monthly_limit: u64,
    pub is_active: bool,
    pub plan: Plan,
    pub models: Vec<Model>,
    pub period: String,
    pub renews_on: Option<String>,
    #[serde(default)]
    pub remaster_model_types: Vec<RemasterModelInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Plan {
    pub name: String,
    pub plan_key: String,
    #[serde(default)]
    pub usage_plan_features: Vec<Feature>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Feature {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub name: String,
    pub external_key: String,
    pub can_use: bool,
    pub is_default_model: bool,
    pub description: String,
    #[serde(default)]
    pub max_lengths: MaxLengths,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct MaxLengths {
    #[serde(default)]
    pub title: u32,
    #[serde(default)]
    pub prompt: u32,
    #[serde(default)]
    pub tags: u32,
    #[serde(default)]
    pub negative_tags: u32,
    #[serde(default)]
    pub gpt_description_prompt: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemasterModelInfo {
    pub name: String,
    pub external_key: String,
    pub is_default_model: bool,
    pub can_use: bool,
}

// --- Clips / Feed ---

#[derive(Debug, Deserialize, Serialize)]
pub struct Clip {
    pub id: String,
    pub title: String,
    pub status: String,
    pub model_name: String,
    pub audio_url: Option<String>,
    pub video_url: Option<String>,
    pub image_url: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub play_count: u64,
    #[serde(default)]
    pub upvote_count: u64,
    #[serde(default)]
    pub metadata: ClipMetadata,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ClipMetadata {
    pub tags: Option<String>,
    pub prompt: Option<String>,
    pub duration: Option<f64>,
    pub avg_bpm: Option<f64>,
    #[serde(default)]
    pub has_stem: bool,
    #[serde(default)]
    pub is_remix: bool,
    #[serde(default)]
    pub make_instrumental: bool,
    #[serde(rename = "type")]
    pub clip_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedResponse {
    #[serde(default)]
    pub clips: Vec<Clip>,
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

// --- Generation ---

#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub mv: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpt_description_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_tags: Option<String>,
    pub make_instrumental: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_clip_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_at: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weirdness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_influence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation_category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    #[serde(default)]
    pub clips: Vec<Clip>,
    pub status: Option<String>,
}

// --- Lyrics ---

#[derive(Debug, Deserialize)]
pub struct LyricsSubmitResponse {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LyricsResult {
    pub text: String,
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub error_message: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

// --- Concat ---

#[derive(Debug, Serialize)]
pub struct ConcatRequest {
    pub clip_id: String,
}

// --- Cover ---

#[derive(Debug, Serialize)]
pub struct CoverRequest {
    pub clip_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
}

// --- Remaster ---

#[derive(Debug, Serialize)]
pub struct RemasterRequest {
    pub clip_id: String,
    pub remaster_model: String,
}

// --- Stems ---

#[derive(Debug, Serialize)]
pub struct StemsRequest {
    pub clip_id: String,
}
