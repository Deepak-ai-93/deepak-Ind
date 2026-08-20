import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createLeaderboardEntry, fixtureSetFingerprint, leaderboardMarkdown } from "./leaderboard.js";
import type { BenchmarkCase, BenchmarkResult } from "./types.js";

const cases: BenchmarkCase[] = [{ id: "one", task: "inspect", root: ".", expectedFiles: ["b", "a"], budgetTokens: 100 }];
const results: BenchmarkResult[] = [{ id: "one", task: "inspect", baselineInputTokens: 100, selectedInputTokens: 50, tokensSaved: 50, savingsPercent: 50, expectedFiles: ["a", "b"], selectedFiles: ["a", "b"], expectedFilesSelected: 2, relevanceRecall: 100, underBudget: true }];

describe("benchmark leaderboard", () => {
  it("fingerprints fixture definitions stably and scores runs", () => {
    assert.equal(fixtureSetFingerprint(cases), fixtureSetFingerprint([{ id: "one", task: "inspect", root: ".", expectedFiles: ["a", "b"], budgetTokens: 100 }]));
    const entry = createLeaderboardEntry(cases, results, "2026-01-01T00:00:00.000Z");
    assert.equal(entry.score, 75);
    assert.match(leaderboardMarkdown([entry]), /IND Benchmark Leaderboard/);
  });
});

