import { randomUUID } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
function costFor(provider, model, usage) { const rates = { "openai-compatible:gpt-4o-mini": { input: 0.15, output: 0.6 }, "openai:gpt-4o-mini": { input: 0.15, output: 0.6 } }; const rate = rates[`${provider}:${model}`.toLowerCase()]; return rate ? (usage.inputTokens / 1_000_000) * rate.input + (usage.outputTokens / 1_000_000) * rate.output : 0; }
function load(path) { if (!existsSync(path))
    return { sessions: [], events: [] }; try {
    return JSON.parse(readFileSync(path, "utf8"));
}
catch {
    return { sessions: [], events: [] };
} }
export function openUsageLedger(projectRoot) { const directory = join(projectRoot, ".ind"); mkdirSync(directory, { recursive: true }); const path = join(directory, "usage.json"); const data = load(path); const save = () => writeFileSync(path, JSON.stringify(data, null, 2), "utf8"); return { startSession(provider, model) { const id = randomUUID(); data.sessions.push({ id, provider, model, startedAt: new Date().toISOString(), status: "active" }); save(); return id; }, finishSession(sessionId, status) { const session = data.sessions.find((item) => item.id === sessionId); if (session) {
        session.status = status;
        session.endedAt = new Date().toISOString();
        save();
    } }, recordEvent(event) { data.events.push({ ...event, id: randomUUID(), createdAt: new Date().toISOString() }); save(); }, summary(sessionId) { const events = sessionId ? data.events.filter((event) => event.sessionId === sessionId) : data.events; const inputTokens = events.reduce((sum, event) => sum + event.usage.inputTokens, 0); const outputTokens = events.reduce((sum, event) => sum + event.usage.outputTokens, 0); const cachedTokens = events.reduce((sum, event) => sum + (event.usage.cachedTokens ?? 0), 0); const baselineInputTokens = events.reduce((sum, event) => sum + (event.baselineInputTokens ?? 0), 0); return { sessions: new Set(events.map((event) => event.sessionId)).size, events: events.length, inputTokens, outputTokens, cachedTokens, totalTokens: events.reduce((sum, event) => sum + event.usage.totalTokens, 0), estimatedCost: events.reduce((sum, event) => sum + (event.estimatedCost ?? costFor(event.provider, event.model, event.usage)), 0), baselineInputTokens, tokensSaved: Math.max(0, baselineInputTokens - inputTokens), averageLatencyMs: events.length ? events.reduce((sum, event) => sum + event.latencyMs, 0) / events.length : 0 }; }, close() { } }; }
//# sourceMappingURL=ledger.js.map