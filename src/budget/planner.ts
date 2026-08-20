import type { IndConfig } from "../config.js";
import type { ContextSelection } from "../context/selector.js";
import type { RouteDecision } from "../routing/router.js";
import type { TaskPlan } from "../tasks/types.js";

export interface BudgetPlan {
  task: string;
  provider: string;
  model: string;
  tier: RouteDecision["tier"];
  contextTokens: number;
  baselineInputTokens: number;
  estimatedInputTokens: number;
  estimatedOutputTokens: number;
  estimatedToolTurns: number;
  estimatedTotalTokens: number;
  estimatedCost: number;
  estimatedSavingsPercent: number;
  warnings: string[];
}

const KNOWN_RATES: Record<string, { input: number; output: number }> = {
  "openai-compatible:gpt-4o-mini": { input: 0.15, output: 0.6 },
  "openai:gpt-4o-mini": { input: 0.15, output: 0.6 },
};

function estimateCost(provider: string, model: string, inputTokens: number, outputTokens: number): number {
  const rate = KNOWN_RATES[`${provider}:${model}`.toLowerCase()];
  if (!rate) return 0;
  return (inputTokens / 1_000_000) * rate.input + (outputTokens / 1_000_000) * rate.output;
}

function estimateBaseline(context: ContextSelection): number {
  return context.selected.reduce((total, file) => total + file.estimatedTokens, 0) + context.omitted.reduce((total, file) => total + Math.max(1, file.relativePath.length / 4), 0);
}

function outputAllowance(config: IndConfig, kind: RouteDecision["kind"]): number {
  if (kind === "summarize" || kind === "inspect") return Math.min(config.maxOutputTokens, 1_000);
  if (kind === "verify") return Math.min(config.maxOutputTokens, 1_500);
  return config.maxOutputTokens;
}

export function createBudgetPlan(config: IndConfig, context: ContextSelection, taskPlan: TaskPlan, route: RouteDecision): BudgetPlan {
  const promptOverhead = Math.max(128, Math.ceil(taskPlan.task.length / 2) + taskPlan.chunks.length * 96);
  const estimatedInputTokens = context.estimatedTokens + promptOverhead;
  const estimatedOutputTokens = outputAllowance(config, route.kind);
  const estimatedToolTurns = Math.min(config.maxToolTurns, taskPlan.chunks.length);
  const estimatedTotalTokens = estimatedInputTokens + estimatedOutputTokens;
  const baselineInputTokens = Math.max(estimatedInputTokens, Math.ceil(estimateBaseline(context)));
  const estimatedSavingsPercent = baselineInputTokens === 0 ? 0 : Math.max(0, ((baselineInputTokens - estimatedInputTokens) / baselineInputTokens) * 100);
  const warnings: string[] = [];
  if (!route.model) warnings.push("No model is configured; cost is shown as zero until a provider model is selected.");
  if (context.omitted.length > 0) warnings.push(`${context.omitted.length} repository files were omitted by relevance or budget limits.`);
  if (estimatedInputTokens > config.maxInputTokens) warnings.push("Estimated prompt exceeds the configured input budget.");
  return {
    task: taskPlan.task,
    provider: config.provider,
    model: route.model,
    tier: route.tier,
    contextTokens: context.estimatedTokens,
    baselineInputTokens,
    estimatedInputTokens,
    estimatedOutputTokens,
    estimatedToolTurns,
    estimatedTotalTokens,
    estimatedCost: estimateCost(config.provider, route.model, estimatedInputTokens, estimatedOutputTokens),
    estimatedSavingsPercent,
    warnings,
  };
}
