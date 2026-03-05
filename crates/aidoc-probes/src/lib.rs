use aidoc_core::{EnvironmentSnapshot, NetworkProbe, ProxySnapshot, ToolVersion};
use aidoc_sandbox::{run_readonly, CommandSpec};

pub fn is_elevated() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        false
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

pub fn collect_snapshot(network_on: bool, command_timeout_ms: u64) -> EnvironmentSnapshot {
    let mut snapshot = base_snapshot();
    snapshot.toolchains = collect_toolchains(command_timeout_ms);

    if network_on {
        snapshot.network = collect_network(command_timeout_ms);
    }

    sanitize_snapshot(snapshot)
}

pub fn collect_min_snapshot() -> EnvironmentSnapshot {
    sanitize_snapshot(base_snapshot())
}

fn base_snapshot() -> EnvironmentSnapshot {
    EnvironmentSnapshot {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        shell: read_shell(),
        elevated: is_elevated(),
        path_preview: path_preview(),
        toolchains: Vec::new(),
        proxy: ProxySnapshot {
            http_proxy: read_proxy_var(&["HTTP_PROXY", "http_proxy"]),
            https_proxy: read_proxy_var(&["HTTPS_PROXY", "https_proxy"]),
            no_proxy: read_proxy_var(&["NO_PROXY", "no_proxy"]),
        },
        network: Vec::new(),
    }
}

pub fn sanitize_snapshot(mut snapshot: EnvironmentSnapshot) -> EnvironmentSnapshot {
    snapshot.path_preview = snapshot
        .path_preview
        .into_iter()
        .map(|segment| mask_user_segment(&segment))
        .collect();
    snapshot.proxy.http_proxy = snapshot.proxy.http_proxy.map(|v| mask_proxy_value(&v));
    snapshot.proxy.https_proxy = snapshot.proxy.https_proxy.map(|v| mask_proxy_value(&v));
    snapshot.proxy.no_proxy = snapshot.proxy.no_proxy.map(|v| mask_proxy_value(&v));
    snapshot
}

fn read_shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .or_else(|| std::env::var("ComSpec").ok())
}

fn path_preview() -> Vec<String> {
    let path = std::env::var("PATH").unwrap_or_default();
    let sep = if cfg!(windows) { ';' } else { ':' };

    path.split(sep)
        .filter(|x| !x.trim().is_empty())
        .map(mask_user_segment)
        .take(10)
        .collect()
}

fn mask_user_segment(segment: &str) -> String {
    let unix_mask = "/home/";
    let windows_mask = "\\Users\\";

    if let Some(idx) = segment.find(unix_mask) {
        let mut out = segment.to_string();
        let start = idx + unix_mask.len();
        let rest = &segment[start..];
        let user_end = rest.find('/').unwrap_or(rest.len());
        out.replace_range(start..start + user_end, "[REDACTED_USER]");
        return out;
    }

    if let Some(idx) = segment.find(windows_mask) {
        let mut out = segment.to_string();
        let start = idx + windows_mask.len();
        let rest = &segment[start..];
        let user_end = rest.find('\\').unwrap_or(rest.len());
        out.replace_range(start..start + user_end, "[REDACTED_USER]");
        return out;
    }

    segment.to_string()
}

fn read_proxy_var(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| std::env::var(key).ok())
        .map(|v| mask_proxy_value(&v))
}

fn mask_proxy_value(value: &str) -> String {
    if let Some(schema_idx) = value.find("://") {
        let prefix = &value[..schema_idx + 3];
        let rest = &value[schema_idx + 3..];
        if let Some(at_idx) = rest.find('@') {
            let host_part = &rest[at_idx + 1..];
            return format!("{prefix}[REDACTED_CREDENTIALS]@{host_part}");
        }
    }
    value.to_string()
}

fn collect_toolchains(command_timeout_ms: u64) -> Vec<ToolVersion> {
    let probes = vec![
        ("python", vec!["--version"]),
        ("python3", vec!["--version"]),
        ("pip", vec!["--version"]),
        ("pip3", vec!["--version"]),
        ("node", vec!["-v"]),
        ("npm", vec!["-v"]),
        ("java", vec!["-version"]),
        ("go", vec!["version"]),
        ("rustc", vec!["-V"]),
        ("cargo", vec!["-V"]),
        ("git", vec!["--version"]),
        ("docker", vec!["--version"]),
        ("gcc", vec!["--version"]),
        ("g++", vec!["--version"]),
        ("cc", vec!["--version"]),
        ("c++", vec!["--version"]),
        ("clang", vec!["--version"]),
        ("clang++", vec!["--version"]),
        ("cmake", vec!["--version"]),
        ("make", vec!["--version"]),
        ("ninja", vec!["--version"]),
    ];

    let mut result = Vec::new();
    for (name, args) in probes {
        let spec = CommandSpec {
            program: name.to_string(),
            args: args.iter().map(|v| (*v).to_string()).collect(),
            timeout_ms: command_timeout_ms.max(100),
            max_lines: 3,
        };

        if let Ok(output) = run_readonly(&spec) {
            let line = if !output.stdout.trim().is_empty() {
                output
                    .stdout
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            } else if !output.stderr.trim().is_empty() {
                output
                    .stderr
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            };

            result.push(ToolVersion {
                name: name.to_string(),
                version: line,
            });
        }
    }

    result
}

fn collect_network(command_timeout_ms: u64) -> Vec<NetworkProbe> {
    let targets = [
        "https://pypi.org",
        "https://registry.npmjs.org",
        "https://github.com",
    ];
    let mut result = Vec::new();

    if cfg!(windows) {
        for target in targets {
            result.push(NetworkProbe {
                target: target.to_string(),
                status: "skip_on_windows_m1".to_string(),
            });
        }
        return result;
    }

    for target in targets {
        let spec = CommandSpec {
            program: "curl".to_string(),
            args: vec!["-I".to_string(), target.to_string()],
            timeout_ms: command_timeout_ms.max(100),
            max_lines: 5,
        };

        match run_readonly(&spec) {
            Ok(out) if out.status == 0 => result.push(NetworkProbe {
                target: target.to_string(),
                status: "ok".to_string(),
            }),
            Ok(out) => result.push(NetworkProbe {
                target: target.to_string(),
                status: format!("http_probe_failed_status_{}", out.status),
            }),
            Err(err) => result.push(NetworkProbe {
                target: target.to_string(),
                status: format!("probe_error_{}", err),
            }),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{mask_proxy_value, mask_user_segment};

    #[test]
    fn should_mask_proxy_credentials() {
        let masked = mask_proxy_value("http://alice:secret@proxy.local:8080");
        assert_eq!(masked, "http://[REDACTED_CREDENTIALS]@proxy.local:8080");
    }

    #[test]
    fn should_keep_proxy_without_credentials() {
        let masked = mask_proxy_value("http://proxy.local:8080");
        assert_eq!(masked, "http://proxy.local:8080");
    }

    #[test]
    fn should_mask_unix_user_segment() {
        let masked = mask_user_segment("/home/alice/.cargo/bin");
        assert_eq!(masked, "/home/[REDACTED_USER]/.cargo/bin");
    }
}
