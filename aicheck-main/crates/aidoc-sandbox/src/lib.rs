use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

use thiserror::Error;
use wait_timeout::ChildExt;

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub max_lines: usize,
}

#[derive(Clone, Debug)]
pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    Macos,
    Unknown,
}

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("command not allowed: {0}")]
    CommandNotAllowed(String),
    #[error("unsafe argument detected: {0}")]
    UnsafeArgument(String),
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    #[error("command timeout")]
    Timeout,
    #[error("command wait failed: {0}")]
    WaitFailed(String),
}

#[derive(Clone, Copy)]
struct AllowedCommand {
    program: &'static str,
    arg_prefix: &'static [&'static str],
}

const BANNED_ARGS: &[&str] = &[
    "install",
    "uninstall",
    "remove",
    "delete",
    "setx",
    "add",
    "chmod",
    "chown",
    "mkfs",
    "format",
];

const ALLOWED_LINUX: &[AllowedCommand] = &[
    AllowedCommand {
        program: "uname",
        arg_prefix: &["-a"],
    },
    AllowedCommand {
        program: "lsb_release",
        arg_prefix: &["-a"],
    },
    AllowedCommand {
        program: "env",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "printenv",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "which",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "ldd",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "ss",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "ip",
        arg_prefix: &["a"],
    },
    AllowedCommand {
        program: "resolvectl",
        arg_prefix: &["status"],
    },
    AllowedCommand {
        program: "curl",
        arg_prefix: &["-I"],
    },
    AllowedCommand {
        program: "git",
        arg_prefix: &["config", "--list", "--show-origin"],
    },
    AllowedCommand {
        program: "pip",
        arg_prefix: &["config", "list"],
    },
    AllowedCommand {
        program: "pip",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "pip3",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "npm",
        arg_prefix: &["config", "list"],
    },
    AllowedCommand {
        program: "python",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "python3",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "gcc",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "g++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "cc",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "c++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "clang",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "clang++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "cmake",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "make",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "ninja",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "node",
        arg_prefix: &["-v"],
    },
    AllowedCommand {
        program: "npm",
        arg_prefix: &["-v"],
    },
    AllowedCommand {
        program: "java",
        arg_prefix: &["-version"],
    },
    AllowedCommand {
        program: "go",
        arg_prefix: &["version"],
    },
    AllowedCommand {
        program: "rustc",
        arg_prefix: &["-V"],
    },
    AllowedCommand {
        program: "cargo",
        arg_prefix: &["-V"],
    },
    AllowedCommand {
        program: "git",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "docker",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "id",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "whoami",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "ls",
        arg_prefix: &["-ld"],
    },
];

const ALLOWED_MACOS: &[AllowedCommand] = &[
    AllowedCommand {
        program: "sw_vers",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "scutil",
        arg_prefix: &["--proxy"],
    },
    AllowedCommand {
        program: "xcode-select",
        arg_prefix: &["-p"],
    },
    AllowedCommand {
        program: "codesign",
        arg_prefix: &["-dv"],
    },
    AllowedCommand {
        program: "brew",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "brew",
        arg_prefix: &["config"],
    },
    AllowedCommand {
        program: "curl",
        arg_prefix: &["-I"],
    },
    AllowedCommand {
        program: "python",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "python3",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "pip",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "pip3",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "gcc",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "g++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "cc",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "c++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "clang",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "clang++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "cmake",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "make",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "ninja",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "node",
        arg_prefix: &["-v"],
    },
    AllowedCommand {
        program: "npm",
        arg_prefix: &["-v"],
    },
    AllowedCommand {
        program: "java",
        arg_prefix: &["-version"],
    },
    AllowedCommand {
        program: "go",
        arg_prefix: &["version"],
    },
    AllowedCommand {
        program: "rustc",
        arg_prefix: &["-V"],
    },
    AllowedCommand {
        program: "cargo",
        arg_prefix: &["-V"],
    },
    AllowedCommand {
        program: "git",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "docker",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "which",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "printenv",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "whoami",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "ls",
        arg_prefix: &["-ld"],
    },
];

const ALLOWED_WINDOWS: &[AllowedCommand] = &[
    AllowedCommand {
        program: "where",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "whoami",
        arg_prefix: &["/all"],
    },
    AllowedCommand {
        program: "ipconfig",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "netsh",
        arg_prefix: &["winhttp", "show", "proxy"],
    },
    AllowedCommand {
        program: "systeminfo",
        arg_prefix: &[],
    },
    AllowedCommand {
        program: "python",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "python3",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "pip",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "pip3",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "gcc",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "g++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "clang",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "clang++",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "cmake",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "make",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "ninja",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "node",
        arg_prefix: &["-v"],
    },
    AllowedCommand {
        program: "npm",
        arg_prefix: &["-v"],
    },
    AllowedCommand {
        program: "java",
        arg_prefix: &["-version"],
    },
    AllowedCommand {
        program: "go",
        arg_prefix: &["version"],
    },
    AllowedCommand {
        program: "rustc",
        arg_prefix: &["-V"],
    },
    AllowedCommand {
        program: "cargo",
        arg_prefix: &["-V"],
    },
    AllowedCommand {
        program: "git",
        arg_prefix: &["--version"],
    },
    AllowedCommand {
        program: "docker",
        arg_prefix: &["--version"],
    },
];

pub fn current_platform() -> Platform {
    match std::env::consts::OS {
        "windows" => Platform::Windows,
        "linux" => Platform::Linux,
        "macos" => Platform::Macos,
        _ => Platform::Unknown,
    }
}

pub fn validate_readonly(spec: &CommandSpec) -> Result<(), SandboxError> {
    for arg in &spec.args {
        let lowered = arg.to_ascii_lowercase();
        if BANNED_ARGS.iter().any(|x| *x == lowered) {
            return Err(SandboxError::UnsafeArgument(arg.clone()));
        }
    }

    let allowed = match current_platform() {
        Platform::Windows => ALLOWED_WINDOWS,
        Platform::Linux => ALLOWED_LINUX,
        Platform::Macos => ALLOWED_MACOS,
        Platform::Unknown => {
            return Err(SandboxError::CommandNotAllowed(format!(
                "unsupported platform for {}",
                spec.program
            )))
        }
    };

    for cmd in allowed {
        if spec.program.eq_ignore_ascii_case(cmd.program)
            && args_prefix_match(&spec.args, cmd.arg_prefix)
        {
            return Ok(());
        }
    }

    Err(SandboxError::CommandNotAllowed(format!(
        "{} {}",
        spec.program,
        spec.args.join(" ")
    )))
}

pub fn run_readonly(spec: &CommandSpec) -> Result<CommandResult, SandboxError> {
    validate_readonly(spec)?;

    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SandboxError::SpawnFailed(e.to_string()))?;

    let timeout = Duration::from_millis(spec.timeout_ms.max(1));
    let wait_result = child
        .wait_timeout(timeout)
        .map_err(|e| SandboxError::WaitFailed(e.to_string()))?;

    let status = if let Some(status) = wait_result {
        status
    } else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(SandboxError::Timeout);
    };

    let mut stdout_raw = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout
            .read_to_string(&mut stdout_raw)
            .map_err(|e| SandboxError::WaitFailed(e.to_string()))?;
    }

    let mut stderr_raw = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        stderr
            .read_to_string(&mut stderr_raw)
            .map_err(|e| SandboxError::WaitFailed(e.to_string()))?;
    }

    let (stdout, out_truncated) = truncate_lines(&stdout_raw, spec.max_lines);
    let (stderr, err_truncated) = truncate_lines(&stderr_raw, spec.max_lines);

    Ok(CommandResult {
        status: status.code().unwrap_or(-1),
        stdout,
        stderr,
        truncated: out_truncated || err_truncated,
    })
}

fn args_prefix_match(args: &[String], prefix: &[&str]) -> bool {
    if prefix.is_empty() {
        return true;
    }
    if args.len() < prefix.len() {
        return false;
    }

    args.iter()
        .take(prefix.len())
        .zip(prefix.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn truncate_lines(input: &str, max_lines: usize) -> (String, bool) {
    let mut lines: Vec<&str> = input.lines().collect();
    let truncated = lines.len() > max_lines;
    if truncated {
        lines.truncate(max_lines);
    }
    (lines.join("\n"), truncated)
}

#[cfg(test)]
mod tests {
    use super::{validate_readonly, CommandSpec, SandboxError};

    #[test]
    fn should_reject_unknown_program() {
        let spec = CommandSpec {
            program: "cat".to_string(),
            args: vec!["/etc/passwd".to_string()],
            timeout_ms: 1000,
            max_lines: 10,
        };

        let err = validate_readonly(&spec).expect_err("cat should be denied");
        assert!(matches!(err, SandboxError::CommandNotAllowed(_)));
    }

    #[test]
    fn should_reject_banned_argument() {
        let spec = CommandSpec {
            program: "npm".to_string(),
            args: vec!["install".to_string()],
            timeout_ms: 1000,
            max_lines: 10,
        };

        let err = validate_readonly(&spec).expect_err("banned args should be denied");
        assert!(matches!(err, SandboxError::UnsafeArgument(_)));
    }
}
