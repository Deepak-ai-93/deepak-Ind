import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { join } from "node:path";
import { loadBenchmarkCases, runBenchmark, toMarkdown } from "./runner.js";

describe("context benchmark", () => {
  it("compares selected context with a full-context baseline", async () => {
    const cases = await loadBenchmarkCases(join(process.cwd(), "fixtures", "benchmark"));
    const results = await runBenchmark(cases);
    assert.equal(results.length, 2);
    assert.ok(results.every((result) => result.underBudget));
    assert.ok(results.every((result) => result.savingsPercent > 0));
    assert.ok(results.every((result) => result.relevanceRecall >= 50));
    assert.match(toMarkdown(results), /Total saved/);
  });
});
