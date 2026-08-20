import { readdir, readFile } from "node:fs/promises";
import { join, relative } from "node:path";
const IGNORED = new Set([".git", ".ind", ".agents", "node_modules", "dist", "output"]);
const SECRET_PATTERNS = [
    [/\bsk-[A-Za-z0-9_-]{20,}\b/, "possible OpenAI secret"],
    [/\bAKIA[0-9A-Z]{16}\b/, "possible AWS access key"],
    [/(?:OPENAI_API_KEY|ANTHROPIC_API_KEY|GOOGLE_GENERATIVE_AI_API_KEY)\s*[:=]\s*[^\s#]{8,}/, "provider secret assigned in source"],
    [/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/, "private key material"],
];
export async function scanProjectSecurity(root) {
    const issues = [];
    async function walk(directory) {
        for (const entry of await readdir(directory, { withFileTypes: true })) {
            if (entry.isDirectory()) {
                if (!IGNORED.has(entry.name))
                    await walk(join(directory, entry.name));
                continue;
            }
            if (!entry.isFile() || entry.name === ".env.example")
                continue;
            const absolute = join(directory, entry.name);
            let content;
            try {
                content = await readFile(absolute, "utf8");
            }
            catch {
                continue;
            }
            if (content.includes("\u0000") || content.length > 1_000_000)
                continue;
            content.split(/\r?\n/).forEach((line, index) => {
                for (const [pattern, message] of SECRET_PATTERNS)
                    if (pattern.test(line))
                        issues.push({ file: relative(root, absolute).replaceAll("\\", "/"), line: index + 1, message });
            });
        }
    }
    await walk(root);
    return issues;
}
//# sourceMappingURL=scanner.js.map