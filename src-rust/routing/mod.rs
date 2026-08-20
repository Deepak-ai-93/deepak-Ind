use std::collections::HashSet;

use crate::config::{IndConfig, RoutingMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskKind {
    Summarize,
    Inspect,
    Edit,
    Verify,
    Reasoning,
}

impl TaskKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskKind::Summarize => "summarize",
            TaskKind::Inspect => "inspect",
            TaskKind::Edit => "edit",
            TaskKind::Verify => "verify",
            TaskKind::Reasoning => "reasoning",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub kind: TaskKind,
    pub model: String,
    pub tier: String,
    pub reason: String,
}

const PATTERNS: &[(TaskKind, &str, &str)] = &[
    (
        TaskKind::Summarize,
        r"summari[sz]|explain|describe|list|title|classif",
        "short transformation or classification",
    ),
    (
        TaskKind::Inspect,
        r"find|search|where|inspect|trace|look through|understand",
        "repository inspection task",
    ),
    (
        TaskKind::Verify,
        r"test|verify|lint|typecheck|check|reproduce|debug",
        "verification or diagnosis task",
    ),
    (
        TaskKind::Edit,
        r"add|change|update|remove|refactor|implement|fix|create|write",
        "code modification task",
    ),
];

pub fn classify_task(task: &str) -> TaskKind {
    for (kind, pattern, _) in PATTERNS {
        let re = regex::Regex::new(&format!("(?i){pattern}")).unwrap();
        if re.is_match(task) {
            return *kind;
        }
    }
    TaskKind::Reasoning
}

pub fn route_task(task: &str, config: &IndConfig) -> RouteDecision {
    let kind = classify_task(task);
    if config.routing == RoutingMode::Off || config.cheap_model.is_empty() || config.strong_model.is_empty()
    {
        return RouteDecision {
            kind,
            model: config.model.clone(),
            tier: "configured".to_string(),
            reason: "automatic routing is disabled or tier models are not configured".to_string(),
        };
    }
    let cheap_kinds: HashSet<TaskKind> =
        [TaskKind::Summarize, TaskKind::Inspect].into_iter().collect();
    if cheap_kinds.contains(&kind) {
        let reason = PATTERNS
            .iter()
            .find(|(candidate, _, _)| *candidate == kind)
            .map(|(_, _, r)| *r)
            .unwrap_or("low-complexity task");
        return RouteDecision {
            kind,
            model: config.cheap_model.clone(),
            tier: "cheap".to_string(),
            reason: reason.to_string(),
        };
    }
    RouteDecision {
        kind,
        model: config.strong_model.clone(),
        tier: "strong".to_string(),
        reason: "modification, verification, or reasoning task".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndConfig;
    use std::path::PathBuf;

    fn config_with(routing: RoutingMode, cheap: &str, strong: &str) -> IndConfig {
        IndConfig {
            project_root: PathBuf::from("."),
            provider: "openai-compatible".to_string(),
            model: String::new(),
            cheap_model: cheap.to_string(),
            strong_model: strong.to_string(),
            routing,
            base_url: None,
            approval: crate::config::ApprovalMode::Chunk,
            max_input_tokens: 12_000,
            max_output_tokens: 4_000,
            max_tool_turns: 8,
        }
    }

    #[test]
    fn classifies_task_kinds() {
        assert_eq!(classify_task("Summarize the changes"), TaskKind::Summarize);
        assert_eq!(classify_task("find where the bug is"), TaskKind::Inspect);
        assert_eq!(classify_task("implement the feature"), TaskKind::Edit);
        assert_eq!(classify_task("verify the build passes"), TaskKind::Verify);
        assert_eq!(classify_task("fix the failing test"), TaskKind::Verify);
        assert_eq!(classify_task("ponder the universe"), TaskKind::Reasoning);
    }

    #[test]
    fn routes_cheap_and_strong_tiers() {
        let cfg = config_with(RoutingMode::Auto, "cheap-model", "strong-model");
        let cheap = route_task("explain the config", &cfg);
        assert_eq!(cheap.tier, "cheap");
        assert_eq!(cheap.model, "cheap-model");
        let strong = route_task("implement a new feature", &cfg);
        assert_eq!(strong.tier, "strong");
        assert_eq!(strong.model, "strong-model");
    }

    #[test]
    fn routing_off_uses_configured_model() {
        let cfg = config_with(RoutingMode::Off, "cheap-model", "strong-model");
        let decision = route_task("explain the config", &cfg);
        assert_eq!(decision.tier, "configured");
    }
}