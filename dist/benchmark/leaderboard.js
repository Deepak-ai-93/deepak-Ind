import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
function rounded(value) { return Math.round(value * 100) / 100; }
export function fixtureSetFingerprint(cases) {
    const stable = cases.map(({ id, task, expectedFiles, budgetTokens }) => ({ id, task, expectedFiles: [...expectedFiles].sort(), budgetTokens })).sort((left, right) => left.id.localeCompare(right.id));
    return createHash("sha256").update(JSON.stringify(stable)).digest("hex").slice(0, 12);
}
export function createLeaderboardEntry(cases, results, now = new Date().toISOString()) {
    if (results.length === 0)
        throw new Error("Cannot create a leaderboard entry without benchmark results.");
    const averageSavingsPercent = rounded(results.reduce((sum, result) => sum + result.savingsPercent, 0) / results.length);
    const averageRecallPercent = rounded(results.reduce((sum, result) => sum + result.relevanceRecall, 0) / results.length);
    const underBudgetPercent = rounded(results.filter((result) => result.underBudget).length / results.length * 100);
    const score = rounded(averageSavingsPercent * 0.5 + averageRecallPercent * 0.4 + underBudgetPercent * 0.1);
    const runId = createHash("sha256").update(`${fixtureSetFingerprint(cases)}:${JSON.stringify(results)}`).digest("hex").slice(0, 16);
    return { runId, fixtureSet: fixtureSetFingerprint(cases), cases: results.length, averageSavingsPercent, averageRecallPercent, underBudgetPercent, totalTokensSaved: results.reduce((sum, result) => sum + result.tokensSaved, 0), score, createdAt: now };
}
export function leaderboardMarkdown(entries) {
    const sorted = [...entries].sort((left, right) => right.score - left.score || left.runId.localeCompare(right.runId));
    const lines = ["# IND Benchmark Leaderboard", "", "Runs are ranked by 50% savings, 40% relevance recall, and 10% budget compliance.", "", "| Rank | Run | Fixture set | Score | Savings | Recall | Under budget | Tokens saved |", "|---:|---|---|---:|---:|---:|---:|---:|"];
    sorted.forEach((entry, index) => lines.push(`| ${index + 1} | ${entry.runId} | ${entry.fixtureSet} | ${entry.score}% | ${entry.averageSavingsPercent}% | ${entry.averageRecallPercent}% | ${entry.underBudgetPercent}% | ${entry.totalTokensSaved} |`));
    return `${lines.join("\n")}\n`;
}
export async function writeLeaderboard(outputRoot, entry) {
    await mkdir(outputRoot, { recursive: true });
    const jsonl = join(outputRoot, "leaderboard.jsonl");
    const markdown = join(outputRoot, "leaderboard.md");
    let entries = [];
    try {
        entries = (await readFile(jsonl, "utf8")).split(/\r?\n/).filter(Boolean).map((line) => JSON.parse(line));
    }
    catch { /* first run */ }
    entries = [...entries.filter((item) => item.runId !== entry.runId), entry];
    await writeFile(jsonl, entries.map((item) => JSON.stringify(item)).join("\n") + "\n", "utf8");
    await writeFile(markdown, leaderboardMarkdown(entries), "utf8");
    return { jsonl, markdown };
}
//# sourceMappingURL=leaderboard.js.map