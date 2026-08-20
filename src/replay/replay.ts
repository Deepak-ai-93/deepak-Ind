import type { IndConfig } from "../config.js";
import type { ContextSelection } from "../context/selector.js";
import { createBudgetPlan, type BudgetPlan } from "../budget/planner.js";
import { classifyTask } from "../routing/router.js";
import type { TaskPlan } from "../tasks/types.js";

export interface ReplayCandidate {
  provider: string;
  model: string;
}

export interface ReplayResult extends BudgetPlan {
  candidate: ReplayCandidate;
  rank: number;
}

export function parseReplayCandidates(spec: string | undefined, config: IndConfig): ReplayCandidate[] {
  const fallback = [
    { provider: config.provider, model: config.model },
    ...(config.cheapModel ? [{ provider: config.provider, model: config.cheapModel }] : []),
    ...(config.strongModel ? [{ provider: config.provider, model: config.strongModel }] : []),
  ];
  const candidates = (spec?.split(",") ?? []).map((entry) => {
    const [provider, ...modelParts] = entry.trim().split(":");
    return { provider: provider?.trim() ?? "", model: modelParts.join(":").trim() };
  }).filter((candidate) => candidate.provider && candidate.model);
  const unique = new Map<string, ReplayCandidate>();
  for (const candidate of (candidates.length ? candidates : fallback)) unique.set(`${candidate.provider}:${candidate.model}`, candidate);
  return [...unique.values()];
}

export function replayTask(config: IndConfig, context: ContextSelection, taskPlan: TaskPlan, candidates: ReplayCandidate[]): ReplayResult[] {
  const kind = classifyTask(taskPlan.task);
  return candidates.map((candidate) => {
    const candidateConfig: IndConfig = { ...config, provider: candidate.provider, model: candidate.model, routing: "off" };
    const budget = createBudgetPlan(candidateConfig, context, taskPlan, { kind, model: candidate.model, tier: "configured", reason: "replay candidate" });
    return { ...budget, candidate, rank: 0 };
  }).sort((left, right) => left.estimatedCost - right.estimatedCost || left.estimatedTotalTokens - right.estimatedTotalTokens || left.model.localeCompare(right.model)).map((result, index) => ({ ...result, rank: index + 1 }));
}
