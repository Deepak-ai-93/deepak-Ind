import { readRepositoryFile } from "./repository.js";
const STOP_WORDS = new Set(["a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "in", "is", "it", "of", "on", "or", "that", "the", "to", "with"]);
function terms(task) {
    return [...new Set(task.toLowerCase().split(/[^a-z0-9_/-]+/).flatMap((term) => term.split(/[\/_-]+/)).filter((term) => term.length > 2 && !STOP_WORDS.has(term)))];
}
function estimateTokens(content) {
    return Math.max(1, Math.ceil(content.length / 4));
}
function scoreFile(file, taskTerms) {
    const path = file.relativePath.toLowerCase();
    const basename = path.split("/").at(-1) ?? path;
    const documentation = file.extension === ".md" || file.extension === ".txt";
    let score = file.isSource && !documentation ? 2 : 0;
    const matches = taskTerms.filter((term) => path.includes(term));
    score += matches.length * 12;
    if (basename.includes("test") || basename.includes("spec"))
        score += 1;
    if (path === "package.json" || path === "tsconfig.json" || path.endsWith("/README.md"))
        score += 5;
    if (path === "package-lock.json" || path.endsWith("/package-lock.json") || path === "pack-plan.json")
        score -= 20;
    if (file.bytes > 100_000)
        score -= 8;
    return { score, reason: matches.length ? `path matches: ${matches.join(", ")}` : file.isSource ? "source/config file" : "non-source metadata" };
}
export async function selectContext(snapshot, task, budgetTokens) {
    if (!Number.isInteger(budgetTokens) || budgetTokens <= 0)
        throw new Error("Context budget must be a positive integer.");
    const taskTerms = terms(task);
    const ranked = snapshot.files.map((file) => ({ file, ...scoreFile(file, taskTerms) })).sort((left, right) => right.score - left.score || left.file.relativePath.localeCompare(right.file.relativePath));
    const selected = [];
    const omitted = [];
    let total = 0;
    for (const candidate of ranked) {
        if (candidate.score <= 0) {
            omitted.push({ relativePath: candidate.file.relativePath, reason: "not relevant to task or supported source" });
            continue;
        }
        const content = await readRepositoryFile(candidate.file);
        if (content === undefined) {
            omitted.push({ relativePath: candidate.file.relativePath, reason: candidate.file.isSource ? "binary, unreadable, or too large" : "not source content" });
            continue;
        }
        const estimated = estimateTokens(content);
        if (total + estimated > budgetTokens) {
            omitted.push({ relativePath: candidate.file.relativePath, reason: `budget exceeded (${estimated} estimated tokens)` });
            continue;
        }
        selected.push({ relativePath: candidate.file.relativePath, content, estimatedTokens: estimated, score: candidate.score, reason: candidate.reason });
        total += estimated;
    }
    return { task, budgetTokens, estimatedTokens: total, selected, omitted };
}
//# sourceMappingURL=selector.js.map