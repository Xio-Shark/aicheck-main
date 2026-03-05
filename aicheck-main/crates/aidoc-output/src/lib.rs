use aidoc_core::Pack;

pub fn render_pack_md(pack: &Pack) -> String {
    let mut out = String::new();

    out.push_str("# AIDOC Handover Pack\n\n");
    out.push_str(&format!("Trace ID: `{}`\n\n", pack.trace_id));
    out.push_str("## 1) Issue Summary\n\n");
    out.push_str(&format!("{}\n\n", pack.issue_summary));

    out.push_str("## 2) Repro Steps\n\n");
    for (idx, step) in pack.repro_steps.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", idx + 1, step));
    }
    out.push('\n');

    out.push_str("## 3) Key Error Evidence\n\n");
    if pack.key_error_evidence.is_empty() {
        out.push_str("- (empty)\n\n");
    } else {
        out.push_str("```text\n");
        for line in &pack.key_error_evidence {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("```\n\n");
    }

    out.push_str("## 4) Environment Snapshot\n\n");
    out.push_str(&format!(
        "- OS: {}\n- Arch: {}\n- Shell: {}\n- Elevated: {}\n\n",
        pack.environment_snapshot.os,
        pack.environment_snapshot.arch,
        pack.environment_snapshot
            .shell
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        pack.environment_snapshot.elevated
    ));

    if !pack.environment_snapshot.toolchains.is_empty() {
        out.push_str("| Tool | Version |\n|---|---|\n");
        for item in &pack.environment_snapshot.toolchains {
            out.push_str(&format!("| {} | {} |\n", item.name, item.version));
        }
        out.push('\n');
    }

    if !pack.environment_snapshot.path_preview.is_empty() {
        out.push_str("PATH preview:\n");
        for segment in &pack.environment_snapshot.path_preview {
            out.push_str(&format!("- {}\n", segment));
        }
        out.push('\n');
    }

    out.push_str("Proxy:\n");
    out.push_str(&format!(
        "- HTTP_PROXY: {}\n- HTTPS_PROXY: {}\n- NO_PROXY: {}\n\n",
        pack.environment_snapshot
            .proxy
            .http_proxy
            .clone()
            .unwrap_or_else(|| "(not set)".to_string()),
        pack.environment_snapshot
            .proxy
            .https_proxy
            .clone()
            .unwrap_or_else(|| "(not set)".to_string()),
        pack.environment_snapshot
            .proxy
            .no_proxy
            .clone()
            .unwrap_or_else(|| "(not set)".to_string())
    ));

    out.push_str("## 5) Root Cause Candidates (Ranked)\n\n");
    if pack.candidates.is_empty() {
        out.push_str("- No ranked candidates from current signatures.\n\n");
    } else {
        for (idx, candidate) in pack.candidates.iter().enumerate() {
            out.push_str(&format!(
                "{}. **{}** (`{}`) confidence `{:.2}`\n",
                idx + 1,
                candidate.title,
                candidate.id,
                candidate.confidence
            ));
            out.push_str(&format!("   - Cause: {}\n", candidate.cause));
            if !candidate.evidence.is_empty() {
                out.push_str("   - Evidence:\n");
                for e in &candidate.evidence {
                    out.push_str(&format!("     - {}\n", e));
                }
            }
            if !candidate.verify_commands.is_empty() {
                out.push_str("   - Verify commands:\n");
                for cmd in &candidate.verify_commands {
                    out.push_str(&format!("     - `{}`\n", cmd));
                }
            }
            if !candidate.fix_suggestions.is_empty() {
                out.push_str("   - Fix suggestions:\n");
                for suggestion in &candidate.fix_suggestions {
                    out.push_str(&format!("     - {}\n", suggestion));
                }
            }
            out.push('\n');
        }
    }

    out.push_str("## 6) Ask for Advanced Model\n\n");
    out.push_str("Please provide the shortest repair plan with verification points, avoid reinstall unless strictly required, and keep cross-platform notes for Windows/Linux/macOS.\n\n");

    out.push_str("## 7) Redaction Report\n\n");
    if pack.redactions.is_empty() {
        out.push_str("- No sensitive pattern was redacted.\n\n");
    } else {
        for item in &pack.redactions {
            out.push_str(&format!(
                "- {} -> {} (count: {})\n",
                item.kind, item.placeholder, item.count
            ));
        }
        out.push('\n');
    }

    out.push_str(&format!("Generated at: `{}`\n", pack.generated_at));
    out
}

pub fn render_pack_json(pack: &Pack) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(pack)
}

#[cfg(test)]
mod tests {
    use aidoc_core::{EnvironmentSnapshot, Pack, ProxySnapshot};

    use super::{render_pack_json, render_pack_md};

    fn sample_pack() -> Pack {
        Pack {
            trace_id: "aidoc-1-2".to_string(),
            issue_summary: "summary".to_string(),
            repro_steps: vec!["step".to_string()],
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
    fn md_should_include_trace_id() {
        let md = render_pack_md(&sample_pack());
        assert!(md.contains("Trace ID: `aidoc-1-2`"));
    }

    #[test]
    fn json_should_include_trace_id() {
        let json = render_pack_json(&sample_pack()).expect("json render should succeed");
        assert!(json.contains("\"trace_id\": \"aidoc-1-2\""));
    }
}
