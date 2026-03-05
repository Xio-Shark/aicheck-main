use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use aidoc_core::Pack;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";
const DEFAULT_OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL_OLLAMA: &str = "qwen2.5:7b";
const DEFAULT_MODEL_OPENAI: &str = "gpt-4o-mini";
const DEFAULT_MODEL_ANTHROPIC: &str = "claude-3-5-sonnet-latest";

const SYSTEM_PROMPT: &str = r#"你是一个环境诊断摘要助手。用户会提供一份脱敏后的诊断报告。
你的任务：
1. 用 3-5 句话总结核心问题
2. 从候选根因中挑出最可能的 1-2 个，解释原因
3. 给出最短修复路径（优先不重装）
禁止：编造不存在的证据、假设未提供的上下文。
修复命令如涉及系统修改，必须标注“⚠️ 潜在修改系统”。"#;

#[derive(Clone, Debug, Default)]
pub struct LlmRequestOptions {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub timeout_ms: u64,
    pub dry_run: bool,
}

#[derive(Clone, Debug)]
pub struct LlmExecution {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub summary: Option<String>,
    pub payload_preview: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn summarize(&self, pack: &Pack) -> Result<String>;
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    fn endpoint(&self) -> &str;
    fn dry_run_payload(&self, pack: &Pack) -> Result<Value>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ProviderKind {
    Ollama,
    OpenAICompat,
    Anthropic,
    Custom,
}

#[derive(Clone, Debug)]
struct ProviderConfig {
    kind: ProviderKind,
    model: String,
    endpoint: String,
    api_key: Option<String>,
    timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct FileConfig {
    provider: Option<FileProvider>,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct FileProvider {
    #[serde(rename = "type")]
    provider_type: Option<String>,
    model: Option<String>,
    endpoint: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OllamaProvider {
    model: String,
    endpoint: String,
    timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct OpenAICompatProvider {
    model: String,
    endpoint: String,
    api_key: String,
    timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct AnthropicProvider {
    model: String,
    endpoint: String,
    api_key: String,
    timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct CustomEndpointProvider {
    model: String,
    endpoint: String,
    api_key: Option<String>,
    timeout_ms: u64,
}

pub async fn summarize_pack(pack: &Pack, options: &LlmRequestOptions) -> Result<LlmExecution> {
    let cfg = resolve_config(options)?;
    let provider = build_provider(&cfg)?;
    let payload = provider.dry_run_payload(pack)?;
    let payload_preview = serde_json::to_string_pretty(&payload)?;
    let identity = provider_identity(provider.as_ref());

    if options.dry_run {
        return Ok(build_execution(identity, None, payload_preview));
    }

    let summary = provider.summarize(pack).await?;
    Ok(build_execution(identity, Some(summary), payload_preview))
}

fn build_execution(
    identity: (String, String, String),
    summary: Option<String>,
    payload_preview: String,
) -> LlmExecution {
    LlmExecution {
        provider: identity.0,
        model: identity.1,
        endpoint: identity.2,
        summary,
        payload_preview,
    }
}

fn provider_identity(provider: &dyn LlmProvider) -> (String, String, String) {
    (
        provider.name().to_string(),
        provider.model().to_string(),
        provider.endpoint().to_string(),
    )
}

fn resolve_config(options: &LlmRequestOptions) -> Result<ProviderConfig> {
    let file_provider = load_file_provider()?;
    let provider_raw = pick_string(
        options.provider.clone(),
        env::var("AIDOC_LLM_PROVIDER").ok(),
        file_provider
            .as_ref()
            .and_then(|cfg| cfg.provider_type.clone()),
        Some("ollama".to_string()),
    );

    let kind = ProviderKind::parse(provider_raw.as_deref().unwrap_or("ollama"))?;
    let model_default = default_model(&kind);
    let endpoint_default = default_endpoint(&kind);
    let model = pick_string(
        options.model.clone(),
        env::var("AIDOC_LLM_MODEL").ok(),
        file_provider.as_ref().and_then(|cfg| cfg.model.clone()),
        Some(model_default.to_string()),
    )
    .unwrap_or_else(|| model_default.to_string());

    let endpoint_raw = pick_string(
        options.endpoint.clone(),
        env::var("AIDOC_LLM_ENDPOINT").ok(),
        file_provider.as_ref().and_then(|cfg| cfg.endpoint.clone()),
        Some(endpoint_default.to_string()),
    )
    .unwrap_or_else(|| endpoint_default.to_string());

    let endpoint = normalize_endpoint(&kind, &endpoint_raw);
    let api_key = pick_string(
        options.api_key.clone(),
        env::var("AIDOC_API_KEY").ok(),
        None,
        None,
    );
    let timeout_ms = effective_timeout(options.timeout_ms);

    if matches!(kind, ProviderKind::OpenAICompat | ProviderKind::Anthropic) && api_key.is_none() {
        return Err(anyhow!("provider {} requires AIDOC_API_KEY", kind.as_str()));
    }

    Ok(ProviderConfig {
        kind,
        model,
        endpoint,
        api_key,
        timeout_ms,
    })
}

fn build_provider(cfg: &ProviderConfig) -> Result<Box<dyn LlmProvider>> {
    match cfg.kind {
        ProviderKind::Ollama => Ok(Box::new(OllamaProvider {
            model: cfg.model.clone(),
            endpoint: cfg.endpoint.clone(),
            timeout_ms: cfg.timeout_ms,
        })),
        ProviderKind::OpenAICompat => Ok(Box::new(OpenAICompatProvider {
            model: cfg.model.clone(),
            endpoint: cfg.endpoint.clone(),
            api_key: cfg
                .api_key
                .clone()
                .ok_or_else(|| anyhow!("missing api key for openai-compatible provider"))?,
            timeout_ms: cfg.timeout_ms,
        })),
        ProviderKind::Anthropic => Ok(Box::new(AnthropicProvider {
            model: cfg.model.clone(),
            endpoint: cfg.endpoint.clone(),
            api_key: cfg
                .api_key
                .clone()
                .ok_or_else(|| anyhow!("missing api key for anthropic provider"))?,
            timeout_ms: cfg.timeout_ms,
        })),
        ProviderKind::Custom => Ok(Box::new(CustomEndpointProvider {
            model: cfg.model.clone(),
            endpoint: cfg.endpoint.clone(),
            api_key: cfg.api_key.clone(),
            timeout_ms: cfg.timeout_ms,
        })),
    }
}

fn pick_string(
    first: Option<String>,
    second: Option<String>,
    third: Option<String>,
    fallback: Option<String>,
) -> Option<String> {
    first
        .or(second)
        .or(third)
        .or(fallback)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn effective_timeout(timeout_ms: u64) -> u64 {
    if timeout_ms == 0 {
        DEFAULT_TIMEOUT_MS
    } else {
        timeout_ms.max(1000)
    }
}

fn default_model(kind: &ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Ollama => DEFAULT_MODEL_OLLAMA,
        ProviderKind::OpenAICompat | ProviderKind::Custom => DEFAULT_MODEL_OPENAI,
        ProviderKind::Anthropic => DEFAULT_MODEL_ANTHROPIC,
    }
}

fn default_endpoint(kind: &ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Ollama => DEFAULT_OLLAMA_ENDPOINT,
        ProviderKind::OpenAICompat | ProviderKind::Custom => DEFAULT_OPENAI_ENDPOINT,
        ProviderKind::Anthropic => DEFAULT_ANTHROPIC_ENDPOINT,
    }
}

fn normalize_endpoint(kind: &ProviderKind, raw: &str) -> String {
    let trimmed = raw.trim_end_matches('/');

    match kind {
        ProviderKind::Ollama => append_if_missing(trimmed, "/api/chat"),
        ProviderKind::OpenAICompat | ProviderKind::Custom => {
            if trimmed.ends_with("/chat/completions") {
                trimmed.to_string()
            } else if trimmed.ends_with("/v1") {
                format!("{trimmed}/chat/completions")
            } else {
                append_if_missing(trimmed, "/v1/chat/completions")
            }
        }
        ProviderKind::Anthropic => append_if_missing(trimmed, "/v1/messages"),
    }
}

fn append_if_missing(base: &str, suffix: &str) -> String {
    if base.ends_with(suffix) {
        base.to_string()
    } else {
        format!("{base}{suffix}")
    }
}

fn load_file_provider() -> Result<Option<FileProvider>> {
    let path = match resolve_config_file_path() {
        Some(path) => path,
        None => return Ok(None),
    };

    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let config: FileConfig = toml::from_str(&text)
        .with_context(|| format!("failed to parse toml: {}", path.display()))?;
    Ok(config.provider)
}

fn resolve_config_file_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let appdata = env::var("APPDATA").ok()?;
        Some(PathBuf::from(appdata).join("aidoc").join("llm.toml"))
    }

    #[cfg(not(windows))]
    {
        if let Ok(xdg_home) = env::var("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg_home).join("aidoc").join("llm.toml"));
        }

        let home = env::var("HOME").ok()?;
        Some(
            PathBuf::from(home)
                .join(".config")
                .join("aidoc")
                .join("llm.toml"),
        )
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn summarize(&self, pack: &Pack) -> Result<String> {
        let payload = self.request_payload(pack)?;
        let value = post_json(&self.endpoint, &[], &payload, self.timeout_ms)?;
        extract_ollama_text(&value)
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn dry_run_payload(&self, pack: &Pack) -> Result<Value> {
        self.request_payload(pack)
    }
}

impl OllamaProvider {
    fn request_payload(&self, pack: &Pack) -> Result<Value> {
        let input = pack_user_input(pack)?;
        Ok(ollama_payload(&self.model, input))
    }
}

#[async_trait]
impl LlmProvider for OpenAICompatProvider {
    async fn summarize(&self, pack: &Pack) -> Result<String> {
        let payload = self.request_payload(pack)?;
        let headers = [("authorization", format!("Bearer {}", self.api_key.as_str()))];
        let value = post_json(&self.endpoint, &headers, &payload, self.timeout_ms)?;
        extract_openai_text(&value)
    }

    fn name(&self) -> &str {
        "openai_compat"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn dry_run_payload(&self, pack: &Pack) -> Result<Value> {
        self.request_payload(pack)
    }
}

impl OpenAICompatProvider {
    fn request_payload(&self, pack: &Pack) -> Result<Value> {
        let input = pack_user_input(pack)?;
        Ok(openai_payload(&self.model, input))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn summarize(&self, pack: &Pack) -> Result<String> {
        let payload = self.request_payload(pack)?;
        let headers = [
            ("x-api-key", self.api_key.clone()),
            ("anthropic-version", "2023-06-01".to_string()),
        ];
        let value = post_json(&self.endpoint, &headers, &payload, self.timeout_ms)?;
        extract_anthropic_text(&value)
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn dry_run_payload(&self, pack: &Pack) -> Result<Value> {
        self.request_payload(pack)
    }
}

impl AnthropicProvider {
    fn request_payload(&self, pack: &Pack) -> Result<Value> {
        let input = pack_user_input(pack)?;
        Ok(anthropic_payload(&self.model, input))
    }
}

#[async_trait]
impl LlmProvider for CustomEndpointProvider {
    async fn summarize(&self, pack: &Pack) -> Result<String> {
        let payload = self.request_payload(pack)?;
        let mut headers: Vec<(&str, String)> = Vec::new();
        if let Some(key) = &self.api_key {
            headers.push(("authorization", format!("Bearer {}", key)));
        }

        let value = post_json(&self.endpoint, &headers, &payload, self.timeout_ms)?;
        extract_openai_text(&value)
    }

    fn name(&self) -> &str {
        "custom"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn dry_run_payload(&self, pack: &Pack) -> Result<Value> {
        self.request_payload(pack)
    }
}

impl CustomEndpointProvider {
    fn request_payload(&self, pack: &Pack) -> Result<Value> {
        let input = pack_user_input(pack)?;
        Ok(openai_payload(&self.model, input))
    }
}

fn ollama_payload(model: &str, input: String) -> Value {
    json!({
        "model": model,
        "stream": false,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": input}
        ]
    })
}

fn openai_payload(model: &str, input: String) -> Value {
    json!({
        "model": model,
        "temperature": 0.2,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": input}
        ]
    })
}

fn anthropic_payload(model: &str, input: String) -> Value {
    json!({
        "model": model,
        "max_tokens": 600,
        "system": SYSTEM_PROMPT,
        "messages": [{"role": "user", "content": input}]
    })
}

fn post_json(
    endpoint: &str,
    headers: &[(&str, String)],
    payload: &Value,
    timeout_ms: u64,
) -> Result<Value> {
    let payload_text = payload.to_string();
    let timeout_secs = timeout_ms.div_ceil(1000).max(1).to_string();
    let mut command = Command::new("curl");

    command
        .arg("-sS")
        .arg("--fail-with-body")
        .arg("-X")
        .arg("POST")
        .arg(endpoint)
        .arg("-H")
        .arg("content-type: application/json")
        .arg("--max-time")
        .arg(timeout_secs)
        .arg("-d")
        .arg(payload_text);

    for (key, value) in headers {
        command.arg("-H").arg(format!("{key}: {value}"));
    }

    let output = command.output().context("failed to spawn curl process")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow!(
            "curl request failed: status={} stderr={} body={}",
            output.status,
            stderr.trim(),
            stdout.trim()
        ));
    }

    let body = String::from_utf8(output.stdout).context("provider response is not utf8")?;
    serde_json::from_str(&body).context("failed to parse provider response as json")
}

fn pack_user_input(pack: &Pack) -> Result<String> {
    serde_json::to_string(pack).context("failed to serialize redacted pack")
}

fn extract_ollama_text(value: &Value) -> Result<String> {
    value
        .get("message")
        .and_then(|msg| msg.get("content"))
        .and_then(|content| content.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| anyhow!("ollama response missing message.content"))
}

fn extract_openai_text(value: &Value) -> Result<String> {
    let content = value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .ok_or_else(|| anyhow!("openai-compatible response missing choices[0].message.content"))?;

    if let Some(text) = content.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if let Some(parts) = content.as_array() {
        let merged = join_text_parts(parts);
        if !merged.is_empty() {
            return Ok(merged);
        }
    }

    Err(anyhow!("openai-compatible response content is empty"))
}

fn extract_anthropic_text(value: &Value) -> Result<String> {
    let content = value
        .get("content")
        .and_then(|items| items.as_array())
        .ok_or_else(|| anyhow!("anthropic response missing content[]"))?;

    let merged = join_text_parts(content);

    if merged.is_empty() {
        Err(anyhow!("anthropic response content is empty"))
    } else {
        Ok(merged)
    }
}

fn join_text_parts(parts: &[Value]) -> String {
    parts
        .iter()
        .filter_map(|part| part.get("text").and_then(|text| text.as_str()))
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

impl ProviderKind {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" | "openai_compat" | "openai-compat" => Ok(Self::OpenAICompat),
            "anthropic" => Ok(Self::Anthropic),
            "custom" => Ok(Self::Custom),
            other => Err(anyhow!("unsupported llm provider: {}", other)),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAICompat => "openai_compat",
            Self::Anthropic => "anthropic",
            Self::Custom => "custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use aidoc_core::{EnvironmentSnapshot, Pack, ProxySnapshot};

    use super::{normalize_endpoint, summarize_pack, LlmRequestOptions, ProviderKind};

    fn sample_pack() -> Pack {
        Pack {
            trace_id: "aidoc-test-trace".to_string(),
            issue_summary: "summary".to_string(),
            repro_steps: vec!["step1".to_string()],
            key_error_evidence: vec!["evidence".to_string()],
            environment_snapshot: EnvironmentSnapshot {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                shell: Some("bash".to_string()),
                elevated: false,
                path_preview: vec!["/usr/bin".to_string()],
                toolchains: Vec::new(),
                proxy: ProxySnapshot::default(),
                network: Vec::new(),
            },
            candidates: Vec::new(),
            redactions: Vec::new(),
            generated_at: "2026-03-03T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn should_normalize_ollama_endpoint() {
        let endpoint = normalize_endpoint(&ProviderKind::Ollama, "http://localhost:11434");
        assert_eq!(endpoint, "http://localhost:11434/api/chat");
    }

    #[test]
    fn should_normalize_openai_endpoint() {
        let endpoint = normalize_endpoint(&ProviderKind::OpenAICompat, "https://api.openai.com/v1");
        assert_eq!(endpoint, "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn should_return_dry_run_payload_without_network_call() {
        let options = LlmRequestOptions {
            provider: Some("ollama".to_string()),
            model: Some("qwen2.5:7b".to_string()),
            endpoint: Some("http://localhost:11434".to_string()),
            api_key: None,
            timeout_ms: 5000,
            dry_run: true,
        };

        let result = futures::executor::block_on(summarize_pack(&sample_pack(), &options))
            .expect("dry-run should succeed");
        assert_eq!(result.provider, "ollama");
        assert!(result.summary.is_none());
        assert!(result.payload_preview.contains("messages"));
    }
}
