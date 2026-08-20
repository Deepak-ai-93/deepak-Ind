import type { IndConfig } from "../config.js";

export type TaskKind = "summarize" | "inspect" | "edit" | "verify" | "reasoning";

export interface RouteDecision {
  kind: TaskKind;
  model: string;
  tier: "cheap" | "strong" | "configured";
  reason: string;
}

const patterns: Array<[TaskKind, RegExp, string]> = [
  ["summarize", /summari[sz]|explain|describe|list|title|classif/i, "short transformation or classification"],
  ["inspect", /find|search|where|inspect|trace|look through|understand/i, "repository inspection task"],
  ["verify", /test|verify|lint|typecheck|check|reproduce|debug/i, "verification or diagnosis task"],
  ["edit", /add|change|update|remove|refactor|implement|fix|create|write/i, "code modification task"],
];

export function classifyTask(task: string): TaskKind {
  for (const [kind, pattern] of patterns) if (pattern.test(task)) return kind;
  return "reasoning";
}

export function routeTask(task: string, config: IndConfig): RouteDecision {
  const kind = classifyTask(task);
  if (config.routing === "off" || !config.cheapModel || !config.strongModel) return { kind, model: config.model, tier: "configured", reason: "automatic routing is disabled or tier models are not configured" };
  const cheapKinds = new Set<TaskKind>(["summarize", "inspect"]);
  if (cheapKinds.has(kind)) return { kind, model: config.cheapModel, tier: "cheap", reason: patterns.find(([candidate]) => candidate === kind)?.[2] ?? "low-complexity task" };
  return { kind, model: config.strongModel, tier: "strong", reason: "modification, verification, or reasoning task" };
}
