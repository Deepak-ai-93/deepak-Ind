use std::fs;
use std::path::{Path, PathBuf};

use colored::*;
use regex::Regex;

/// A single diagnostic finding from the security scanner.
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub severity: Severity,
    pub category: &'static str,
    pub message: String,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Pass,
    Info,
    Warning,
    Critical,
}

impl Severity {
    pub fn label(&self) -> ColoredString {
        match self {
            Severity::Pass => "PASS".green().bold(),
            Severity::Info => "INFO".blue().bold(),
            Severity::Warning => "WARN".yellow().bold(),
            Severity::Critical => "CRIT".red().bold(),
        }
    }
}

/// Full diagnostic report produced by `ind doctor`.
#[derive(Debug)]
pub struct DiagnosticReport {
    pub findings: Vec<SecurityFinding>,
}

impl DiagnosticReport {
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let mut pass = 0;
        let mut info = 0;
        let mut warn = 0;
        let mut crit = 0;
        for f in &self.findings {
            match f.severity {
                Severity::Pass => pass += 1,
                Severity::Info => info += 1,
                Severity::Warning => warn += 1,
                Severity::Critical => crit += 1,
            }
        }
        (pass, info, warn, crit)
    }

    pub fn is_healthy(&self) -> bool {
        self.findings
            .iter()
            .all(|f| f.severity != Severity::Critical)
    }
}

// ---------------------------------------------------------------------------
// Secret patterns — high-confidence regex patterns for leaked credentials.
// ---------------------------------------------------------------------------

struct SecretPattern {
    name: &'static str,
    regex: &'static str,
}

const SECRET_PATTERNS: &[SecretPattern] = &[
    SecretPattern {
        name: "AWS Access Key",
        regex: r"(?i)AKIA[0-9A-Z]{16}",
    },
    SecretPattern {
        name: "AWS Secret Key",
        regex: r"(?i)aws[_\-]?secret[_\-]?access[_\-]?key\s*[:=]\s*[A-Za-z0-9/+=]{30,}",
    },
    SecretPattern {
        name: "Generic API Key assignment",
        regex: r#"(?i)(api[_\-]?key|apikey|secret[_\-]?key|access[_\-]?token)\s*[:=]\s*["']?[A-Za-z0-9\-_]{20,}["']?"#,
    },
    SecretPattern {
        name: "Private Key block",
        regex: r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
    },
    SecretPattern {
        name: "GitHub Token",
        regex: r"gh[pousr]_[A-Za-z0-9_]{36,}",
    },
    SecretPattern {
        name: "Slack Token",
        regex: r"xox[bporas]-[0-9]+-[0-9]+-[A-Za-z0-9]+",
    },
    SecretPattern {
        name: "Generic Bearer Token",
        regex: r#"(?i)bearer\s+[A-Za-z0-9\-_\.]{20,}"#,
    },
];

// ---------------------------------------------------------------------------
// Files to check for .gitignore coverage.
// ---------------------------------------------------------------------------

const SENSITIVE_FILENAMES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    ".env.staging",
    "credentials.json",
    "service-account.json",
    "secrets.json",
    "id_rsa",
    "id_ed25519",
];

// ---------------------------------------------------------------------------
// Scan implementation.
// ---------------------------------------------------------------------------

/// Run all security checks against the given project root.
pub fn scan_project(project_root: &Path) -> DiagnosticReport {
    let mut findings = Vec::new();

    check_gitignore_coverage(project_root, &mut findings);
    check_env_files_tracked(project_root, &mut findings);
    check_secret_leaks(project_root, &mut findings);
    check_ind_config_permissions(project_root, &mut findings);
    check_policy_present(project_root, &mut findings);

    DiagnosticReport { findings }
}

// ---------------------------------------------------------------------------
// Check 1: .gitignore covers sensitive patterns.
// ---------------------------------------------------------------------------

fn check_gitignore_coverage(root: &Path, findings: &mut Vec<SecurityFinding>) {
    let gitignore_path = root.join(".gitignore");
    if !gitignore_path.exists() {
        findings.push(SecurityFinding {
            severity: Severity::Warning,
            category: "gitignore",
            message: "No .gitignore found — sensitive files may be committed.".to_string(),
            file: None,
        });
        return;
    }

    let content = match fs::read_to_string(&gitignore_path) {
        Ok(c) => c,
        Err(_) => {
            findings.push(SecurityFinding {
                severity: Severity::Warning,
                category: "gitignore",
                message: "Could not read .gitignore".to_string(),
                file: Some(".gitignore".to_string()),
            });
            return;
        }
    };

    let patterns_to_check = [".env", ".env.*", "*.pem", "*.key"];
    let mut missing = Vec::new();

    for pattern in &patterns_to_check {
        let found = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == *pattern
                || trimmed.starts_with(pattern)
                || (pattern == &".env" && trimmed == ".env*")
                || (pattern == &".env.*" && trimmed == ".env*")
        });
        if !found {
            missing.push(*pattern);
        }
    }

    if missing.is_empty() {
        findings.push(SecurityFinding {
            severity: Severity::Pass,
            category: "gitignore",
            message: ".gitignore covers common secret file patterns.".to_string(),
            file: Some(".gitignore".to_string()),
        });
    } else {
        findings.push(SecurityFinding {
            severity: Severity::Warning,
            category: "gitignore",
            message: format!(".gitignore missing patterns: {}", missing.join(", ")),
            file: Some(".gitignore".to_string()),
        });
    }
}

// ---------------------------------------------------------------------------
// Check 2: Sensitive files that actually exist in the project tree.
// ---------------------------------------------------------------------------

fn check_env_files_tracked(root: &Path, findings: &mut Vec<SecurityFinding>) {
    let mut found_any = false;
    for name in SENSITIVE_FILENAMES {
        let path = root.join(name);
        if path.exists() {
            found_any = true;
            // Check if it's tracked by git (if git exists).
            let is_tracked = is_git_tracked(root, name);
            if is_tracked {
                findings.push(SecurityFinding {
                    severity: Severity::Critical,
                    category: "secrets-file",
                    message: format!(
                        "Sensitive file '{}' is tracked by git — remove it and add to .gitignore.",
                        name
                    ),
                    file: Some(name.to_string()),
                });
            } else {
                findings.push(SecurityFinding {
                    severity: Severity::Info,
                    category: "secrets-file",
                    message: format!(
                        "Sensitive file '{}' exists but is not tracked by git.",
                        name
                    ),
                    file: Some(name.to_string()),
                });
            }
        }
    }

    if !found_any {
        findings.push(SecurityFinding {
            severity: Severity::Pass,
            category: "secrets-file",
            message: "No sensitive credential files found in project root.".to_string(),
            file: None,
        });
    }
}

fn is_git_tracked(root: &Path, file: &str) -> bool {
    // Use `git ls-files` to check if a file is tracked.
    std::process::Command::new("git")
        .args(["ls-files", "--error-unmatch", file])
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Check 3: Scan source files for hardcoded secrets.
// ---------------------------------------------------------------------------

fn check_secret_leaks(root: &Path, findings: &mut Vec<SecurityFinding>) {
    let compiled: Vec<(&str, Regex)> = SECRET_PATTERNS
        .iter()
        .filter_map(|sp| Regex::new(sp.regex).ok().map(|re| (sp.name, re)))
        .collect();

    let mut leak_count: usize = 0;
    let max_findings = 20; // Cap to avoid flooding.

    scan_directory_for_secrets(
        root,
        root,
        &compiled,
        findings,
        &mut leak_count,
        max_findings,
    );

    if leak_count == 0 {
        findings.push(SecurityFinding {
            severity: Severity::Pass,
            category: "secret-leak",
            message: "No hardcoded secrets detected in source files.".to_string(),
            file: None,
        });
    }
}

fn scan_directory_for_secrets(
    root: &Path,
    dir: &Path,
    patterns: &[(&str, Regex)],
    findings: &mut Vec<SecurityFinding>,
    leak_count: &mut usize,
    max_findings: usize,
) {
    if *leak_count >= max_findings {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if *leak_count >= max_findings {
            return;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip directories and files that are not interesting.
        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "fixtures"
        {
            continue;
        }

        if path.is_dir() {
            scan_directory_for_secrets(root, &path, patterns, findings, leak_count, max_findings);
            continue;
        }

        // Only scan text-like files by extension.
        if !is_scannable_file(&name) {
            continue;
        }

        // Skip very large files (> 512KB).
        if let Ok(meta) = fs::metadata(&path)
            && meta.len() > 512 * 1024
        {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        for (line_num, line) in content.lines().enumerate() {
            if *leak_count >= max_findings {
                return;
            }
            for (pattern_name, re) in patterns {
                if re.is_match(line) {
                    *leak_count += 1;
                    findings.push(SecurityFinding {
                        severity: Severity::Critical,
                        category: "secret-leak",
                        message: format!(
                            "Possible {} in {}:{}",
                            pattern_name,
                            relative,
                            line_num + 1
                        ),
                        file: Some(relative.clone()),
                    });
                    break; // One finding per line is enough.
                }
            }
        }
    }
}

fn is_scannable_file(name: &str) -> bool {
    let scannable_extensions = [
        "rs", "toml", "json", "yaml", "yml", "ts", "tsx", "js", "jsx", "py", "rb", "go", "java",
        "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd", "cfg", "ini", "conf", "xml", "env",
        "txt", "md", "html", "css", "scss", "sql",
    ];
    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
        scannable_extensions.contains(&ext.to_lowercase().as_str())
    } else {
        // Files without extension: check common names.
        matches!(
            name,
            "Dockerfile" | "Makefile" | "Rakefile" | "Gemfile" | "Procfile"
        )
    }
}

// ---------------------------------------------------------------------------
// Check 4: IND config directory permissions / existence.
// ---------------------------------------------------------------------------

fn check_ind_config_permissions(root: &Path, findings: &mut Vec<SecurityFinding>) {
    let ind_dir = root.join(".ind");
    if !ind_dir.exists() {
        findings.push(SecurityFinding {
            severity: Severity::Info,
            category: "config",
            message: "No .ind/ directory found — using defaults.".to_string(),
            file: None,
        });
        return;
    }

    // Check config.json exists and is valid JSON.
    let config_path = ind_dir.join("config.json");
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => {
                    findings.push(SecurityFinding {
                        severity: Severity::Pass,
                        category: "config",
                        message: ".ind/config.json is valid JSON.".to_string(),
                        file: Some(".ind/config.json".to_string()),
                    });
                }
                Err(e) => {
                    findings.push(SecurityFinding {
                        severity: Severity::Warning,
                        category: "config",
                        message: format!(".ind/config.json is invalid JSON: {e}"),
                        file: Some(".ind/config.json".to_string()),
                    });
                }
            },
            Err(e) => {
                findings.push(SecurityFinding {
                    severity: Severity::Warning,
                    category: "config",
                    message: format!("Could not read .ind/config.json: {e}"),
                    file: Some(".ind/config.json".to_string()),
                });
            }
        }
    }

    // Check usage.db exists.
    let db_path = ind_dir.join("usage.db");
    if db_path.exists() {
        findings.push(SecurityFinding {
            severity: Severity::Pass,
            category: "config",
            message: ".ind/usage.db exists — usage ledger active.".to_string(),
            file: Some(".ind/usage.db".to_string()),
        });
    } else {
        findings.push(SecurityFinding {
            severity: Severity::Info,
            category: "config",
            message: ".ind/usage.db not found — usage ledger not yet created.".to_string(),
            file: Some(".ind/usage.db".to_string()),
        });
    }
}

// ---------------------------------------------------------------------------
// Check 5: Policy file validation.
// ---------------------------------------------------------------------------

fn check_policy_present(root: &Path, findings: &mut Vec<SecurityFinding>) {
    let policy_path = root.join(".ind").join("policy.json");
    if policy_path.exists() {
        match fs::read_to_string(&policy_path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => {
                    findings.push(SecurityFinding {
                        severity: Severity::Pass,
                        category: "policy",
                        message: ".ind/policy.json is valid and loaded.".to_string(),
                        file: Some(".ind/policy.json".to_string()),
                    });
                }
                Err(e) => {
                    findings.push(SecurityFinding {
                        severity: Severity::Warning,
                        category: "policy",
                        message: format!(".ind/policy.json is invalid JSON: {e}"),
                        file: Some(".ind/policy.json".to_string()),
                    });
                }
            },
            Err(e) => {
                findings.push(SecurityFinding {
                    severity: Severity::Warning,
                    category: "policy",
                    message: format!("Could not read .ind/policy.json: {e}"),
                    file: Some(".ind/policy.json".to_string()),
                });
            }
        }
    } else {
        findings.push(SecurityFinding {
            severity: Severity::Info,
            category: "policy",
            message: "No .ind/policy.json found — no team policy restrictions enforced."
                .to_string(),
            file: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Environment diagnostics (for `ind doctor` integration).
// ---------------------------------------------------------------------------

/// Check which provider API keys are available in the environment.
pub fn check_provider_keys() -> Vec<SecurityFinding> {
    let mut findings = Vec::new();
    let keys = [
        ("OPENAI_API_KEY", "OpenAI"),
        ("ANTHROPIC_API_KEY", "Anthropic"),
        ("GOOGLE_GENERATIVE_AI_API_KEY", "Google AI"),
    ];

    for (var, name) in &keys {
        match std::env::var(var) {
            Ok(val) if !val.trim().is_empty() => {
                findings.push(SecurityFinding {
                    severity: Severity::Pass,
                    category: "env-keys",
                    message: format!("{name} API key ({var}) is set."),
                    file: None,
                });
            }
            _ => {
                findings.push(SecurityFinding {
                    severity: Severity::Info,
                    category: "env-keys",
                    message: format!("{name} API key ({var}) is not set."),
                    file: None,
                });
            }
        }
    }

    findings
}

/// Check that essential toolchain binaries are available.
pub fn check_toolchain() -> Vec<SecurityFinding> {
    let mut findings = Vec::new();
    let tools = [("git", "Git version control"), ("cargo", "Rust toolchain")];

    for (bin, label) in &tools {
        let available = std::process::Command::new(bin)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if available {
            findings.push(SecurityFinding {
                severity: Severity::Pass,
                category: "toolchain",
                message: format!("{label} ({bin}) is available."),
                file: None,
            });
        } else {
            findings.push(SecurityFinding {
                severity: Severity::Warning,
                category: "toolchain",
                message: format!("{label} ({bin}) is not installed or not in PATH."),
                file: None,
            });
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_project(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ind-security-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn pass_when_gitignore_covers_secrets() {
        let dir = temp_project("gitignore-pass");
        fs::write(dir.join(".gitignore"), ".env\n.env.*\n*.pem\n*.key\n").unwrap();
        let mut findings = Vec::new();
        check_gitignore_coverage(&dir, &mut findings);
        assert!(findings.iter().any(|f| f.severity == Severity::Pass));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn warn_when_gitignore_missing() {
        let dir = temp_project("gitignore-missing");
        let mut findings = Vec::new();
        check_gitignore_coverage(&dir, &mut findings);
        assert!(findings.iter().any(|f| f.severity == Severity::Warning));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn detects_hardcoded_aws_key() {
        let dir = temp_project("secret-leak");
        fs::write(
            dir.join("config.rs"),
            "let key = \"AKIAIOSFODNN7EXAMPLE\";\n",
        )
        .unwrap();
        let mut findings = Vec::new();
        check_secret_leaks(&dir, &mut findings);
        assert!(
            findings
                .iter()
                .any(|f| f.severity == Severity::Critical && f.message.contains("AWS")),
            "Expected critical finding for AWS key, got: {:?}",
            findings
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pass_when_no_secrets_in_source() {
        let dir = temp_project("no-secrets");
        fs::write(dir.join("main.rs"), "fn main() { println!(\"hello\"); }\n").unwrap();
        let mut findings = Vec::new();
        check_secret_leaks(&dir, &mut findings);
        assert!(findings.iter().any(|f| f.severity == Severity::Pass));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn validates_ind_config_json() {
        let dir = temp_project("config-valid");
        let ind_dir = dir.join(".ind");
        fs::create_dir_all(&ind_dir).unwrap();
        fs::write(ind_dir.join("config.json"), "{\"provider\":\"openai\"}").unwrap();
        let mut findings = Vec::new();
        check_ind_config_permissions(&dir, &mut findings);
        assert!(
            findings
                .iter()
                .any(|f| f.severity == Severity::Pass && f.message.contains("valid JSON"))
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn warns_on_invalid_config_json() {
        let dir = temp_project("config-invalid");
        let ind_dir = dir.join(".ind");
        fs::create_dir_all(&ind_dir).unwrap();
        fs::write(ind_dir.join("config.json"), "not json at all").unwrap();
        let mut findings = Vec::new();
        check_ind_config_permissions(&dir, &mut findings);
        assert!(
            findings
                .iter()
                .any(|f| f.severity == Severity::Warning && f.message.contains("invalid JSON"))
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn full_scan_runs_without_panic() {
        let dir = temp_project("full-scan");
        fs::write(dir.join(".gitignore"), ".env\n").unwrap();
        let report = scan_project(&dir);
        assert!(!report.findings.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn toolchain_check_runs() {
        let findings = check_toolchain();
        // Should always have findings for git and cargo.
        assert_eq!(findings.len(), 2);
    }
}
