use serde::Serialize;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewPayload {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    pub html: String,
    pub css: String,
    pub scope: serde_json::Value,
    #[serde(rename = "moduleUrl")]
    pub module_url: String,
    pub islands: Vec<String>,
}
