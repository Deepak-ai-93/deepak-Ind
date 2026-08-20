import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { openUsageLedger } from "./ledger.js";
import { formatUsageSummary } from "./monitor.js";

describe("usage ledger", () => {
  it("persists sessions, usage events, costs, and baseline savings", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-usage-"));
    const ledger = openUsageLedger(root);
    const session = ledger.startSession("openai-compatible", "gpt-4o-mini");
    ledger.recordEvent({ sessionId: session, provider: "openai-compatible", model: "gpt-4o-mini", eventType: "chat", usage: { inputTokens: 100, outputTokens: 20, totalTokens: 120, cachedTokens: 10, estimated: false }, latencyMs: 250, baselineInputTokens: 180 });
    ledger.finishSession(session, "completed");
    const summary = ledger.summary();
    assert.equal(summary.sessions, 1);
    assert.equal(summary.events, 1);
    assert.equal(summary.inputTokens, 100);
    assert.equal(summary.cachedTokens, 10);
    assert.equal(summary.tokensSaved, 80);
    assert.match(formatUsageSummary(summary), /80 saved/);
    ledger.close();
    const reopened = openUsageLedger(root);
    assert.equal(reopened.summary().totalTokens, 120);
    reopened.close();
  });
});
