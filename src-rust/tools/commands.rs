use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::policy::{IndPolicy, assert_policy_command_allowed, load_policy};

const MAX_OUTPUT_BYTES: usize = 32_000;
const BLOCKED_PATTERNS: &[&str] = &[
    r"\brm\s+-rf\b",
    r"\bdel\s+/s\b",
    r"\bformat\s+[a-z]:",
    r"\bshutdown\b",
    r"\breg\s+delete\b",
];

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub fn assert_command_allowed(command: &str) -> Result<(), String> {
    if command.trim().is_empty() {
        return Err("Command cannot be empty.".to_string());
    }
    for pattern in BLOCKED_PATTERNS {
        let re = regex::Regex::new(pattern).unwrap();
        if re.is_match(command) {
            return Err(format!("Command blocked by IND safety policy: {command}"));
        }
    }
    Ok(())
}

fn shell_command(command: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", command]);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command]);
        cmd
    }
}

fn append_output(current: &str, chunk: &[u8]) -> String {
    let mut next = String::from_utf8_lossy(chunk).to_string();
    next = format!("{current}{next}");
    if next.len() > MAX_OUTPUT_BYTES {
        next = next[next.len() - MAX_OUTPUT_BYTES..].to_string();
    }
    next
}

pub fn run_project_command(
    command: &str,
    cwd: &Path,
    options: &RunCommandOptions,
) -> Result<CommandResult, String> {
    assert_command_allowed(command)?;
    let policy = options
        .policy
        .clone()
        .unwrap_or_else(|| load_policy(cwd).unwrap_or_default());
    assert_policy_command_allowed(command, &policy)?;
    if !options.approved {
        return Err("Command requires explicit approval.".to_string());
    }

    let timeout_ms = options.timeout_ms.unwrap_or(120_000);
    let mut cmd = shell_command(command);
    cmd.current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut timed_out = false;
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let exit_code = status.code();
                if let Some(mut out) = child.stdout.take() {
                    use std::io::Read;
                    let mut buf = Vec::new();
                    let _ = out.read_to_end(&mut buf);
                    stdout = append_output(&stdout, &buf);
                }
                if let Some(mut err) = child.stderr.take() {
                    use std::io::Read;
                    let mut buf = Vec::new();
                    let _ = err.read_to_end(&mut buf);
                    stderr = append_output(&stderr, &buf);
                }
                return Ok(CommandResult {
                    command: command.to_string(),
                    exit_code,
                    stdout,
                    stderr,
                    timed_out,
                });
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(CommandResult {
                        command: command.to_string(),
                        exit_code: None,
                        stdout,
                        stderr,
                        timed_out,
                    });
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("Failed to run command: {e}"));
            }
        }
    }
}

pub struct RunCommandOptions {
    pub timeout_ms: Option<u64>,
    pub approved: bool,
    pub policy: Option<IndPolicy>,
}

impl Default for RunCommandOptions {
    fn default() -> Self {
        Self {
            timeout_ms: None,
            approved: true,
            policy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_destructive_commands() {
        assert!(assert_command_allowed("rm -rf /").is_err());
        assert!(assert_command_allowed("shutdown now").is_err());
        assert!(assert_command_allowed("cargo test").is_ok());
        assert!(assert_command_allowed("").is_err());
    }

    #[test]
    fn rejects_unapproved_commands() {
        let result = run_project_command(
            "echo hi",
            Path::new("."),
            &RunCommandOptions {
                approved: false,
                ..Default::default()
            },
        );
        assert_eq!(result.unwrap_err(), "Command requires explicit approval.");
    }

    #[test]
    fn runs_command_and_captures_output() {
        #[cfg(target_os = "windows")]
        let probe = "Write-Output hello";
        #[cfg(not(target_os = "windows"))]
        let probe = "echo hello";
        let result =
            run_project_command(probe, Path::new("."), &RunCommandOptions::default()).unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.to_lowercase().contains("hello"));
    }

    #[test]
    fn times_out_long_running_commands() {
        #[cfg(target_os = "windows")]
        let probe = "Start-Sleep -Seconds 30";
        #[cfg(not(target_os = "windows"))]
        let probe = "sleep 30";
        let result = run_project_command(
            probe,
            Path::new("."),
            &RunCommandOptions {
                timeout_ms: Some(200),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(result.timed_out);
    }
}
