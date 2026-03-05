use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Path,
    Proxy,
    Cert,
    Permission,
    Toolchain,
    Dependency,
    Version,
    Wsl,
    Network,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleHit {
    pub id: String,
    pub category: Category,
    pub confidence: f32,
    pub title: String,
    pub cause_en: String,
    pub cause_zh: String,
    pub evidence: Vec<String>,
    pub verify_commands: Vec<String>,
    #[serde(default)]
    pub fix_suggestions_en: Vec<String>,
    #[serde(default)]
    pub fix_suggestions_zh: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub category: Category,
    pub confidence: f32,
    pub title: String,
    pub cause: String,
    pub evidence: Vec<String>,
    pub verify_commands: Vec<String>,
    #[serde(default)]
    pub fix_suggestions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RedactionItem {
    pub kind: String,
    pub placeholder: String,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProxySnapshot {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolVersion {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkProbe {
    pub target: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EnvironmentSnapshot {
    pub os: String,
    pub arch: String,
    pub shell: Option<String>,
    pub elevated: bool,
    pub path_preview: Vec<String>,
    pub toolchains: Vec<ToolVersion>,
    pub proxy: ProxySnapshot,
    pub network: Vec<NetworkProbe>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pack {
    pub trace_id: String,
    pub issue_summary: String,
    pub repro_steps: Vec<String>,
    pub key_error_evidence: Vec<String>,
    pub environment_snapshot: EnvironmentSnapshot,
    pub candidates: Vec<Candidate>,
    pub redactions: Vec<RedactionItem>,
    pub generated_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputLanguage {
    En,
    Zh,
}
