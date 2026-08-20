import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { parseReplayCandidates, replayTask } from "./replay.js";
import type { IndConfig } from "../config.js";
import type { ContextSelection } from "../context/selector.js";
import type { TaskPlan } from "../tasks/types.js";

const config: IndConfig = { projectRoot: process.cwd(), provider: "openai-compatible", model: "gpt-4o-mini", cheapModel: "fast", strongModel: "strong", routing: "auto", approval: "chunk", maxInputTokens: 1000, maxOutputTokens: 500, maxToolTurns: 4 };
const context: ContextSelection = { task: "summarize", budgetTokens: 1000, estimatedTokens: 100, selected: [], omitted: [] };
const task: TaskPlan = { id: "task", task: "summarize this module", contextFiles: [], createdAt: new Date().toISOString(), chunks: [] };

describe("replay mode", () => {
  it("parses unique provider/model candidates", () => {
    assert.deepEqual(parseReplayCandidates("openai:gpt-4o-mini,anthropic:claude-3,openai:gpt-4o-mini", config), [{ provider: "openai", model: "gpt-4o-mini" }, { provider: "anthropic", model: "claude-3" }]);
  });

  it("ranks candidates deterministically without network calls", () => {
    const results = replayTask(config, context, task, [{ provider: "openai", model: "gpt-4o-mini" }, { provider: "local", model: "small" }]);
    assert.equal(results.length, 2);
    assert.equal(results[0]?.rank, 1);
    assert.equal(results[0]?.model, "small");
  });
});

