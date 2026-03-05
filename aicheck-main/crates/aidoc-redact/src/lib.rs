use aidoc_core::RedactionItem;
use once_cell::sync::Lazy;
use regex::Regex;

pub struct RedactionResult {
    pub redacted: String,
    pub items: Vec<RedactionItem>,
}

struct RedactionRule {
    kind: &'static str,
    placeholder: &'static str,
    regex: Regex,
}

static RULES: Lazy<Vec<RedactionRule>> = Lazy::new(|| {
    vec![
        RedactionRule {
            kind: "bearer_token",
            placeholder: "Bearer [REDACTED_TOKEN]",
            regex: Regex::new(r"(?i)bearer\s+[a-z0-9._\-]+").expect("invalid bearer regex"),
        },
        RedactionRule {
            kind: "api_key",
            placeholder: "[REDACTED_API_KEY]",
            regex: Regex::new(r"\bsk-[A-Za-z0-9]{16,}\b").expect("invalid api key regex"),
        },
        RedactionRule {
            kind: "github_token",
            placeholder: "[REDACTED_GITHUB_TOKEN]",
            regex: Regex::new(r"\bgh[pousr]_[A-Za-z0-9]{20,}\b")
                .expect("invalid github token regex"),
        },
        RedactionRule {
            kind: "email",
            placeholder: "[REDACTED_EMAIL]",
            regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
                .expect("invalid email regex"),
        },
        RedactionRule {
            kind: "ipv4",
            placeholder: "[REDACTED_IP]",
            regex: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("invalid ipv4 regex"),
        },
        RedactionRule {
            kind: "unix_home_path",
            placeholder: "/home/[REDACTED_USER]",
            regex: Regex::new(r"/home/[A-Za-z0-9._-]+").expect("invalid unix home path regex"),
        },
        RedactionRule {
            kind: "windows_user_path",
            placeholder: "C:\\Users\\[REDACTED_USER]",
            regex: Regex::new(r"(?i)[A-Z]:\\Users\\[^\\\s]+").expect("invalid windows path regex"),
        },
        RedactionRule {
            kind: "private_key_block",
            placeholder: "[REDACTED_PRIVATE_KEY_BLOCK]",
            regex: Regex::new(
                r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
            )
            .expect("invalid private key block regex"),
        },
    ]
});

pub fn redact_text(input: &str) -> RedactionResult {
    let mut redacted = input.to_string();
    let mut items = Vec::new();

    for rule in RULES.iter() {
        let count = rule.regex.find_iter(&redacted).count();
        if count == 0 {
            continue;
        }

        redacted = rule
            .regex
            .replace_all(&redacted, rule.placeholder)
            .into_owned();

        items.push(RedactionItem {
            kind: rule.kind.to_string(),
            placeholder: rule.placeholder.to_string(),
            count,
        });
    }

    RedactionResult { redacted, items }
}

#[cfg(test)]
mod tests {
    use super::redact_text;

    #[test]
    fn should_redact_api_key_and_email() {
        let input = "token sk-1234567890abcdefghijkl and mail test@example.com";
        let out = redact_text(input);
        assert!(out.redacted.contains("[REDACTED_API_KEY]"));
        assert!(out.redacted.contains("[REDACTED_EMAIL]"));
    }
}
