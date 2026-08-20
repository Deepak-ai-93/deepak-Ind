use crate::config::IndConfig;
use crate::context::selector::ContextSelection;
use crate::routing::{RouteDecision, TaskKind};

#[derive(Debug, Clone)]
pub struct BudgetPlan {
    pub task: String,
    pub provider: String,
    pub model: String,
    pub tier: String,
    pub context_tokens: usize,
    pub baseline_input_tokens: usize,
    pub estimated_input_tokens: usize,
    pub estimated_output_tokens: usize,
    pub estimated_tool_turns: usize,
    pub estimated_total_tokens: usize,
    pub estimated_cost: f64,
    pub estimated_savings_percent: f64,
    pub warnings: Vec<String>,
}

const KNOWN_RATES: &[(&str, f64, f64)] = &[
    ("openai-compatible:gpt-4o-mini", 0.15, 0.60),
    ("openai:gpt-4o-mini", 0.15, 0.60),
];

fn estimate_cost(provider: &str, model: &str, input_tokens: usize, output_tokens: usize) -> f64 {
    let key = format!("{provider}:{model}").to_lowercase();
    for (candidate, input_rate, output_rate) in KNOWN_RATES {
        if *candidate == key {
            return (input_tokens as f64 / 1_000_000.0) * input_rate
                + (output_tokens as f64 / 1_000_000.0) * output_rate;
        }
    }
    0.0
}

fn estimate_baseline(context: &ContextSelection) -> usize {
    let selected: usize = context.selected.iter().map(|f| f.estimated_tokens).sum();
    let omitted: usize = context
        .omitted
        .iter()
        .map(|f| f.relative_path.chars().count().div_ceil(4).max(1))
        .sum();
    selected + omitted
}

fn output_allowance(config: &IndConfig, kind: TaskKind) -> usize {
    match kind {
        TaskKind::Summarize | TaskKind::Inspect => config.max_output_tokens.min(1_000),
        TaskKind::Verify => config.max_output_tokens.min(1_500),
        _ => config.max_output_tokens,
    }
}

pub fn create_budget_plan(
    config: &IndConfig,
    context: &ContextSelection,
    task: &str,
    chunk_count: usize,
    route: &RouteDecision,
) -> BudgetPlan {
    let prompt_overhead =
        (task.chars().count().div_ceil(2)).max(128) + chunk_count * 96;
    let estimated_input_tokens = context.estimated_tokens + prompt_overhead;
    let estimated_output_tokens = output_allowance(config, route.kind);
    let estimated_tool_turns = config.max_tool_turns.min(chunk_count);
    let estimated_total_tokens = estimated_input_tokens + estimated_output_tokens;
    let baseline_input_tokens = estimated_input_tokens.max(estimate_baseline(context));
    let estimated_savings_percent = if baseline_input_tokens == 0 {
        0.0
    } else {
        (((baseline_input_tokens - estimated_input_tokens) as f64 / baseline_input_tokens as f64)
            * 100.0)
            .max(0.0)
    };

    let mut warnings = Vec::new();
    if route.model.is_empty() {
        warnings.push(
            "No model is configured; cost is shown as zero until a provider model is selected."
                .to_string(),
        );
    }
    if !context.omitted.is_empty() {
        warnings.push(format!(
            "{} repository files were omitted by relevance or budget limits.",
            context.omitted.len()
        ));
    }
    if estimated_input_tokens > config.max_input_tokens {
        warnings.push("Estimated prompt exceeds the configured input budget.".to_string());
    }

    BudgetPlan {
        task: task.to_string(),
        provider: config.provider.clone(),
        model: route.model.clone(),
        tier: route.tier.clone(),
        context_tokens: context.estimated_tokens,
        baseline_input_tokens,
        estimated_input_tokens,
        estimated_output_tokens,
        estimated_tool_turns,
        estimated_total_tokens,
        estimated_cost: estimate_cost(
            &config.provider,
            &route.model,
            estimated_input_tokens,
            estimated_output_tokens,
        ),
        estimated_savings_percent,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApprovalMode, RoutingMode};
    use crate::context::selector::{ContextSelection, OmittedFile, SelectedContextFile};
    use std::path::PathBuf;

    fn config() -> IndConfig {
        IndConfig {
            project_root: PathBuf::from("."),
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            cheap_model: String::new(),
            strong_model: String::new(),
            routing: RoutingMode::Off,
            base_url: None,
            approval: ApprovalMode::Chunk,
            max_input_tokens: 12_000,
            max_output_tokens: 4_000,
            max_tool_turns: 8,
        }
    }

    fn context(tokens: usize, omitted: usize) -> ContextSelection {
        ContextSelection {
            task: "t".to_string(),
            budget_tokens: 12_000,
            estimated_tokens: tokens,
            selected: vec![SelectedContextFile {
                relative_path: "a.rs".to_string(),
                content: String::new(),
                estimated_tokens: tokens,
                score: 2,
                reason: "source/config file".to_string(),
            }],
            omitted: (0..omitted)
                .map(|i| OmittedFile {
                    relative_path: format!("omitted-{i}.rs"),
                    reason: "budget exceeded".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn budget_plan_reports_warnings() {
        let cfg = config();
        let ctx = context(1_000, 3);
        let route = crate::routing::route_task("write a summary", &cfg);
        let plan = create_budget_plan(&cfg, &ctx, "write a summary", 3, &route);
        assert_eq!(plan.estimated_tool_turns, 3);
        assert!(plan.warnings.iter().any(|w| w.contains("3 repository files")));
    }

    #[test]
    fn cost_uses_known_rates() {
        let cfg = config();
        let ctx = context(1_000_000, 0);
        let route = crate::routing::route_task("edit something", &cfg);
        let plan = create_budget_plan(&cfg, &ctx, "edit something", 3, &route);
        assert!(plan.estimated_cost > 0.0);
    }
}