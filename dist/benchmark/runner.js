import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { inspectRepository, readRepositoryFile } from "../context/repository.js";
import { selectContext } from "../context/selector.js";
function estimate(content) { return Math.max(1, Math.ceil(content.length / 4)); }
export async function loadBenchmarkCases(fixturesRoot) {
    const files = (await readdir(fixturesRoot)).filter((file) => file.endsWith(".json")).sort();
    const cases = [];
    for (const file of files) {
        const parsed = JSON.parse(await readFile(join(fixturesRoot, file), "utf8"));
        cases.push({ ...parsed, root: resolve(fixturesRoot, parsed.root) });
    }
    return cases;
}
export async function runBenchmark(cases) {
    const results = [];
    for (const benchmark of cases) {
        const snapshot = await inspectRepository(benchmark.root);
        let baselineInputTokens = 0;
        for (const file of snapshot.files) {
            const content = await readRepositoryFile(file);
            if (content !== undefined)
                baselineInputTokens += estimate(content);
        }
        const selection = await selectContext(snapshot, benchmark.task, benchmark.budgetTokens);
        const selectedFiles = selection.selected.map((file) => file.relativePath);
        const expectedSelected = benchmark.expectedFiles.filter((file) => selectedFiles.includes(file)).length;
        const tokensSaved = Math.max(0, baselineInputTokens - selection.estimatedTokens);
        results.push({ id: benchmark.id, task: benchmark.task, baselineInputTokens, selectedInputTokens: selection.estimatedTokens, tokensSaved, savingsPercent: baselineInputTokens ? Math.round((tokensSaved / baselineInputTokens) * 10000) / 100 : 0, expectedFiles: benchmark.expectedFiles, selectedFiles, expectedFilesSelected: expectedSelected, relevanceRecall: benchmark.expectedFiles.length ? Math.round((expectedSelected / benchmark.expectedFiles.length) * 10000) / 100 : 1, underBudget: selection.estimatedTokens <= benchmark.budgetTokens });
    }
    return results;
}
export function toJsonl(results) { return results.map((result) => JSON.stringify(result)).join("\n") + (results.length ? "\n" : ""); }
export function toMarkdown(results) {
    const lines = ["# IND Context Benchmark", "", "| Fixture | Baseline tokens | IND tokens | Saved | Savings | Recall | Under budget |", "|---|---:|---:|---:|---:|---:|---|"];
    for (const result of results)
        lines.push(`| ${result.id} | ${result.baselineInputTokens} | ${result.selectedInputTokens} | ${result.tokensSaved} | ${result.savingsPercent}% | ${result.relevanceRecall}% | ${result.underBudget ? "yes" : "no"} |`);
    const totalBaseline = results.reduce((sum, result) => sum + result.baselineInputTokens, 0);
    const totalSelected = results.reduce((sum, result) => sum + result.selectedInputTokens, 0);
    const totalSaved = Math.max(0, totalBaseline - totalSelected);
    lines.push("", `Total baseline: ${totalBaseline} tokens`, `Total IND context: ${totalSelected} tokens`, `Total saved: ${totalSaved} tokens (${totalBaseline ? Math.round((totalSaved / totalBaseline) * 10000) / 100 : 0}%)`);
    return `${lines.join("\n")}\n`;
}
export async function writeBenchmarkReports(outputRoot, results) {
    await mkdir(outputRoot, { recursive: true });
    const jsonl = join(outputRoot, "context-benchmark.jsonl");
    const markdown = join(outputRoot, "context-benchmark.md");
    await writeFile(jsonl, toJsonl(results), "utf8");
    await writeFile(markdown, toMarkdown(results), "utf8");
    return { jsonl, markdown };
}
//# sourceMappingURL=runner.js.map