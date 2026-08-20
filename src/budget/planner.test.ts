import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createBudgetPlan } from "./planner.js";
import type { IndConfig } from "../config.js";
import type { ContextSelection } from "../context/selector.js";
import type { TaskPlan } from "../tasks/types.js";

const config: IndConfig = {
  projectRoot: process.cwd(), provider: "openai-compatible", model: "gpt-4o-mini", cheapModel: "", strongModel: "", routing: "off", approval: "chunk", maxInputTokens: 1_000, maxOutputTokens: 2_000, maxToolTurns: 8,
};
const context: ContextSelection = { task: "fix auth", budgetTokens: 1_000, estimatedTokens: 200, selected: [], omitted: [{ relativePath: "src/auth.ts", reason: "budget" }] };
const taskPlan: TaskPlan = { id: "task", task: "fix auth", contextFiles: [], createdAt: new Date().toISOString(), chunks: [{ id: "1", sequence: 1, title: "one", goal: "one", status: "pending", verification: [] }] };

describe("token budget planner", () => {
  it("estimates bounded output, totals, savings, and known cost", () => {
    const plan = createBudgetPlan(config, context, taskPlan, { kind: "edit", model: "gpt-4o-mini", tier: "configured", reason: "edit" });
    assert.equal(plan.contextTokens, 200);
    assert.equal(plan.estimatedOutputTokens, 2_000);
    assert.equal(plan.estimatedTotalTokens, plan.estimatedInputTokens + plan.estimatedOutputTokens);
    assert.equal(plan.estimatedCost > 0, true);
    assert.equal(plan.warnings.length, 1);
  });
});
