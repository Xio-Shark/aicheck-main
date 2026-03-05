use chrono::Utc;

use crate::model::{Candidate, EnvironmentSnapshot, OutputLanguage, Pack, RedactionItem, RuleHit};

pub fn build_pack(
    redacted_log: &str,
    redactions: Vec<RedactionItem>,
    hits: Vec<RuleHit>,
    environment_snapshot: EnvironmentSnapshot,
    lang: OutputLanguage,
) -> Pack {
    let issue_summary = if hits.is_empty() {
        match lang {
            OutputLanguage::En => {
                "No strong signature matched. Please attach more raw error output for deeper diagnosis."
                    .to_string()
            }
            OutputLanguage::Zh => "未命中强特征规则，请补充更完整的原始报错输出。".to_string(),
        }
    } else {
        let top = &hits[0];
        match lang {
            OutputLanguage::En => format!(
                "The most likely failure category is {:?}. Top signature: {}.",
                top.category, top.id
            ),
            OutputLanguage::Zh => format!(
                "最可能的失败类别是 {:?}，首要命中特征为 {}。",
                top.category, top.id
            ),
        }
    };

    let mut key_error_evidence: Vec<String> = hits
        .iter()
        .flat_map(|hit| hit.evidence.iter().cloned())
        .take(60)
        .collect();

    if key_error_evidence.is_empty() {
        key_error_evidence = redacted_log
            .lines()
            .take(20)
            .map(ToString::to_string)
            .collect();
    }

    let candidates = hits
        .into_iter()
        .map(|hit| Candidate {
            id: hit.id,
            category: hit.category,
            confidence: hit.confidence,
            title: hit.title,
            cause: match lang {
                OutputLanguage::En => hit.cause_en,
                OutputLanguage::Zh => hit.cause_zh,
            },
            evidence: hit.evidence,
            verify_commands: hit.verify_commands,
            fix_suggestions: match lang {
                OutputLanguage::En => hit.fix_suggestions_en,
                OutputLanguage::Zh => hit.fix_suggestions_zh,
            },
        })
        .collect();

    let repro_steps = match lang {
        OutputLanguage::En => vec![
            "Run the original failing command exactly once.".to_string(),
            "Capture full stderr and the last 200 lines of stdout.".to_string(),
            "Run aidoc diagnose and merge both sections via aidoc pack.".to_string(),
        ],
        OutputLanguage::Zh => vec![
            "完整复现一次失败命令。".to_string(),
            "保留完整 stderr 和 stdout 最后 200 行。".to_string(),
            "执行 aidoc diagnose，并使用 aidoc pack 合并诊断。".to_string(),
        ],
    };

    Pack {
        trace_id: build_trace_id(),
        issue_summary,
        repro_steps,
        key_error_evidence,
        environment_snapshot,
        candidates,
        redactions,
        generated_at: Utc::now().to_rfc3339(),
    }
}

fn build_trace_id() -> String {
    let pid = std::process::id();
    let ts = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    format!("aidoc-{pid}-{ts}")
}

#[cfg(test)]
mod tests {
    use crate::{EnvironmentSnapshot, OutputLanguage, RedactionItem, RuleHit};

    use super::build_pack;

    #[test]
    fn should_build_pack_with_trace_id() {
        let pack = build_pack(
            "bash: pip: command not found",
            Vec::<RedactionItem>::new(),
            Vec::<RuleHit>::new(),
            EnvironmentSnapshot::default(),
            OutputLanguage::En,
        );

        assert!(pack.trace_id.starts_with("aidoc-"));
        assert!(!pack.generated_at.is_empty());
    }

    #[test]
    fn should_return_zh_summary_when_requested() {
        let pack = build_pack(
            "error",
            Vec::<RedactionItem>::new(),
            Vec::<RuleHit>::new(),
            EnvironmentSnapshot::default(),
            OutputLanguage::Zh,
        );

        assert!(pack.issue_summary.contains("未命中强特征规则"));
    }
}
