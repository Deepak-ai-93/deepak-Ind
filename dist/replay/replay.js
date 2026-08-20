import { createBudgetPlan } from "../budget/planner.js";
import { classifyTask } from "../routing/router.js";
export function parseReplayCandidates(spec, config) {
    const fallback = [
        { provider: config.provider, model: config.model },
        ...(config.cheapModel ? [{ provider: config.provider, model: config.cheapModel }] : []),
        ...(config.strongModel ? [{ provider: config.provider, model: config.strongModel }] : []),
    ];
    const candidates = (spec?.split(",") ?? []).map((entry) => {
        const [provider, ...modelParts] = entry.trim().split(":");
        return { provider: provider?.trim() ?? "", model: modelParts.join(":").trim() };
    }).filter((candidate) => candidate.provider && candidate.model);
    const unique = new Map();
    for (const candidate of (candidates.length ? candidates : fallback))
        unique.set(`${candidate.provider}:${candidate.model}`, candidate);
    return [...unique.values()];
}
export function replayTask(config, context, taskPlan, candidates) {
    const kind = classifyTask(taskPlan.task);
    return candidates.map((candidate) => {
        const candidateConfig = { ...config, provider: candidate.provider, model: candidate.model, routing: "off" };
        const budget = createBudgetPlan(candidateConfig, context, taskPlan, { kind, model: candidate.model, tier: "configured", reason: "replay candidate" });
        return { ...budget, candidate, rank: 0 };
    }).sort((left, right) => left.estimatedCost - right.estimatedCost || left.estimatedTotalTokens - right.estimatedTotalTokens || left.model.localeCompare(right.model)).map((result, index) => ({ ...result, rank: index + 1 }));
}
//# sourceMappingURL=replay.js.map