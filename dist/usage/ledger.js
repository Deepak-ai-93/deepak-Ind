import { randomUUID } from "node:crypto";
import Database from "better-sqlite3";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
function costFor(provider, model, usage) {
    const key = `${provider}:${model}`.toLowerCase();
    const rates = {
        "openai-compatible:gpt-4o-mini": { input: 0.15, output: 0.6 },
        "openai:gpt-4o-mini": { input: 0.15, output: 0.6 },
    };
    const rate = rates[key];
    if (!rate)
        return 0;
    return ((usage.inputTokens / 1_000_000) * rate.input) + ((usage.outputTokens / 1_000_000) * rate.output);
}
export function openUsageLedger(projectRoot) {
    const directory = join(projectRoot, ".ind");
    mkdirSync(directory, { recursive: true });
    const db = new Database(join(directory, "usage.db"));
    db.pragma("journal_mode = WAL");
    db.exec(`
    CREATE TABLE IF NOT EXISTS sessions (
      id TEXT PRIMARY KEY,
      provider TEXT NOT NULL,
      model TEXT NOT NULL,
      started_at TEXT NOT NULL,
      ended_at TEXT,
      status TEXT NOT NULL DEFAULT 'active'
    );
    CREATE TABLE IF NOT EXISTS usage_events (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      session_id TEXT NOT NULL REFERENCES sessions(id),
      chunk_id TEXT,
      provider TEXT NOT NULL,
      model TEXT NOT NULL,
      event_type TEXT NOT NULL,
      input_tokens INTEGER NOT NULL,
      output_tokens INTEGER NOT NULL,
      cached_tokens INTEGER NOT NULL DEFAULT 0,
      total_tokens INTEGER NOT NULL,
      latency_ms INTEGER NOT NULL,
      estimated_cost REAL NOT NULL DEFAULT 0,
      baseline_input_tokens INTEGER NOT NULL DEFAULT 0,
      created_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_usage_events_session ON usage_events(session_id);
    CREATE INDEX IF NOT EXISTS idx_usage_events_created ON usage_events(created_at);
  `);
    const insertSession = db.prepare("INSERT INTO sessions (id, provider, model, started_at) VALUES (?, ?, ?, ?)");
    const finishSession = db.prepare("UPDATE sessions SET status = ?, ended_at = ? WHERE id = ?");
    const insertEvent = db.prepare(`INSERT INTO usage_events (session_id, chunk_id, provider, model, event_type, input_tokens, output_tokens, cached_tokens, total_tokens, latency_ms, estimated_cost, baseline_input_tokens, created_at) VALUES (@sessionId, @chunkId, @provider, @model, @eventType, @inputTokens, @outputTokens, @cachedTokens, @totalTokens, @latencyMs, @estimatedCost, @baselineInputTokens, @createdAt)`);
    const summaryQuery = db.prepare(`SELECT COUNT(DISTINCT session_id) AS sessions, COUNT(*) AS events, COALESCE(SUM(input_tokens), 0) AS inputTokens, COALESCE(SUM(output_tokens), 0) AS outputTokens, COALESCE(SUM(cached_tokens), 0) AS cachedTokens, COALESCE(SUM(total_tokens), 0) AS totalTokens, COALESCE(SUM(estimated_cost), 0) AS estimatedCost, COALESCE(SUM(baseline_input_tokens), 0) AS baselineInputTokens, COALESCE(AVG(latency_ms), 0) AS averageLatencyMs FROM usage_events ${"WHERE session_id = ?"}`);
    const summaryAllQuery = db.prepare(`SELECT COUNT(DISTINCT session_id) AS sessions, COUNT(*) AS events, COALESCE(SUM(input_tokens), 0) AS inputTokens, COALESCE(SUM(output_tokens), 0) AS outputTokens, COALESCE(SUM(cached_tokens), 0) AS cachedTokens, COALESCE(SUM(total_tokens), 0) AS totalTokens, COALESCE(SUM(estimated_cost), 0) AS estimatedCost, COALESCE(SUM(baseline_input_tokens), 0) AS baselineInputTokens, COALESCE(AVG(latency_ms), 0) AS averageLatencyMs FROM usage_events`);
    return {
        startSession(provider, model) {
            const id = randomUUID();
            insertSession.run(id, provider, model, new Date().toISOString());
            return id;
        },
        finishSession(sessionId, status) { finishSession.run(status, new Date().toISOString(), sessionId); },
        recordEvent(event) {
            insertEvent.run({ sessionId: event.sessionId, chunkId: event.chunkId ?? null, provider: event.provider, model: event.model, eventType: event.eventType, inputTokens: event.usage.inputTokens, outputTokens: event.usage.outputTokens, cachedTokens: event.usage.cachedTokens ?? 0, totalTokens: event.usage.totalTokens, latencyMs: event.latencyMs, estimatedCost: event.estimatedCost ?? costFor(event.provider, event.model, event.usage), baselineInputTokens: event.baselineInputTokens ?? 0, createdAt: new Date().toISOString() });
        },
        summary(sessionId) {
            const raw = (sessionId ? summaryQuery.get(sessionId) : summaryAllQuery.get());
            return { ...raw, tokensSaved: Math.max(0, raw.baselineInputTokens - raw.inputTokens) };
        },
        close() { db.close(); },
    };
}
//# sourceMappingURL=ledger.js.map