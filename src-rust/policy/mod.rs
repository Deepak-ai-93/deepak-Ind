use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::ApprovalMode;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndPolicy {
    pub approval: Option<ApprovalMode>,
    pub allowed_providers: Option<Vec<String>>,
    pub allowed_commands: Option<Vec<String>>,
    pub denied_commands: Option<Vec<String>>,
}

pub fn policy_path(project_root: &Path) -> PathBuf {
    project_root.join(".ind").join("policy.json")
}

fn string_list(value: Option<Vec<String>>, field: &str) -> Result<Option<Vec<String>>, String> {
    match value {
        Some(items) => {
            for item in &items {
                if item.trim().is_empty() {
                    return Err(format!(
                        "Invalid IND policy field '{field}': expected a list of non-empty strings."
                    ));
                }
            }
            Ok(Some(items))
        }
        None => Ok(None),
    }
}

pub fn load_policy(project_root: &Path) -> Result<IndPolicy, String> {
    let path = policy_path(project_root);
    if !path.exists() {
        return Ok(IndPolicy::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("Invalid IND policy at {}: {}", path.display(), e))?;
    let mut parsed: IndPolicy = serde_json::from_str(&text)
        .map_err(|e| format!("Invalid IND policy at {}: {}", path.display(), e))?;

    if let Some(approval) = parsed.approval
        && !matches!(
            approval,
            ApprovalMode::Chunk | ApprovalMode::Command | ApprovalMode::Never
        )
    {
        return Err(
            "Invalid IND policy field 'approval': expected chunk, command, or never.".to_string(),
        );
    }

    parsed.allowed_providers = string_list(parsed.allowed_providers, "allowedProviders")?;
    parsed.allowed_commands = string_list(parsed.allowed_commands, "allowedCommands")?;
    parsed.denied_commands = string_list(parsed.denied_commands, "deniedCommands")?;

    Ok(parsed)
}

fn matches(command: &str, patterns: &[String]) -> Result<bool, String> {
    for pattern in patterns {
        let re = regex::Regex::new(&format!("(?i){}", pattern))
            .map_err(|e| format!("Invalid IND policy command pattern '{pattern}': {e}"))?;
        if re.is_match(command) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn assert_provider_allowed(provider: &str, policy: &IndPolicy) -> Result<(), String> {
    if let Some(allowed) = &policy.allowed_providers
        && !allowed.iter().any(|p| p == provider)
    {
        return Err(format!(
            "Provider '{provider}' is blocked by IND team policy."
        ));
    }
    Ok(())
}

pub fn assert_policy_command_allowed(command: &str, policy: &IndPolicy) -> Result<(), String> {
    if let Some(denied) = &policy.denied_commands
        && matches(command, denied)?
    {
        return Err(format!("Command blocked by IND team policy: {command}"));
    }
    if let Some(allowed) = &policy.allowed_commands
        && !matches(command, allowed)?
    {
        return Err(format!(
            "Command is not on the IND team policy allowlist: {command}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_policy_is_empty() {
        let dir = std::env::temp_dir();
        let policy = load_policy(&dir).unwrap();
        assert!(policy.allowed_providers.is_none());
        assert!(policy.denied_commands.is_none());
    }

    #[test]
    fn policy_blocks_provider() {
        let policy = IndPolicy {
            allowed_providers: Some(vec!["openai".to_string()]),
            ..Default::default()
        };
        assert!(assert_provider_allowed("anthropic", &policy).is_err());
        assert!(assert_provider_allowed("openai", &policy).is_ok());
    }

    #[test]
    fn policy_command_rules() {
        let policy = IndPolicy {
            denied_commands: Some(vec!["rm -rf".to_string()]),
            ..Default::default()
        };
        assert!(assert_policy_command_allowed("rm -rf /", &policy).is_err());
        assert!(assert_policy_command_allowed("cargo test", &policy).is_ok());

        let allowlist = IndPolicy {
            allowed_commands: Some(vec!["^cargo (test|build)$".to_string()]),
            ..Default::default()
        };
        assert!(assert_policy_command_allowed("cargo test", &allowlist).is_ok());
        assert!(assert_policy_command_allowed("npm install", &allowlist).is_err());
    }
}
