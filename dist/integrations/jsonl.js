import { appendFile, readFile } from "node:fs/promises";
import { dirname } from "node:path";
import { mkdir } from "node:fs/promises";
function redact(value) {
    if (typeof value === "string")
        return value.replace(/(api[_-]?key|token|secret|password)\s*[:=]\s*[^\s,}]+/gi, "$1=[REDACTED]").replace(/\bsk-[A-Za-z0-9_-]{12,}\b/g, "[REDACTED]");
    if (Array.isArray(value))
        return value.map(redact);
    if (value && typeof value === "object")
        return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, /key|token|secret|password/i.test(key) ? "[REDACTED]" : redact(item)]));
    return value;
}
export async function appendJsonlEvent(path, event) {
    await mkdir(dirname(path), { recursive: true });
    await appendFile(path, `${JSON.stringify(redact({ ...event, timestamp: event.timestamp ?? new Date().toISOString() }))}\n`, "utf8");
}
export async function readJsonlEvents(path) {
    const content = await readFile(path, "utf8").catch(() => "");
    return content.split(/\r?\n/).filter(Boolean).flatMap((line) => { try {
        return [JSON.parse(line)];
    }
    catch {
        return [];
    } });
}
//# sourceMappingURL=jsonl.js.map