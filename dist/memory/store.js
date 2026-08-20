import Database from "better-sqlite3";
import { appendFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";
const MEMORY_TEMPLATE = `# Memory â€” IND

## Project

- **Goal:** Project-local memory for IND sessions.
- **Where we are:** New project.

## Standing decisions

- Keep memory human-readable and append-only.

## Known lessons & gotchas

- None recorded yet.

## Open questions

- None recorded yet.
`;
function terms(task) {
    return [...new Set(task.toLowerCase().split(/[^a-z0-9_/-]+/).flatMap((part) => part.split(/[\/_-]+/)).filter((part) => part.length > 2))];
}
export function openMemoryStore(projectRoot) {
    const memoryPath = join(projectRoot, "MEMORY.md");
    const directory = join(projectRoot, ".ind");
    mkdirSync(directory, { recursive: true });
    const db = new Database(join(directory, "usage.db"));
    db.exec(`
    CREATE TABLE IF NOT EXISTS memory_entries (
      id TEXT PRIMARY KEY,
      project_root TEXT NOT NULL,
      category TEXT NOT NULL,
      content TEXT NOT NULL,
      source TEXT NOT NULL,
      importance INTEGER NOT NULL DEFAULT 1,
      created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS resume_state (
      project_root TEXT PRIMARY KEY,
      task TEXT NOT NULL,
      session_id TEXT,
      current_chunk INTEGER NOT NULL,
      status TEXT NOT NULL,
      next_step TEXT NOT NULL,
      updated_at TEXT NOT NULL
    );
  `);
    const insertMemory = db.prepare("INSERT INTO memory_entries (id, project_root, category, content, source, importance, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)");
    const upsertResume = db.prepare("INSERT INTO resume_state (project_root, task, session_id, current_chunk, status, next_step, updated_at) VALUES (@projectRoot, @task, @sessionId, @currentChunk, @status, @nextStep, @updatedAt) ON CONFLICT(project_root) DO UPDATE SET task=@task, session_id=@sessionId, current_chunk=@currentChunk, status=@status, next_step=@nextStep, updated_at=@updatedAt");
    const readResume = db.prepare("SELECT task, session_id AS sessionId, current_chunk AS currentChunk, status, next_step AS nextStep, updated_at AS updatedAt FROM resume_state WHERE project_root = ?");
    return {
        async ensureFile() {
            await mkdir(projectRoot, { recursive: true });
            if (!existsSync(memoryPath))
                await writeFile(memoryPath, MEMORY_TEMPLATE, "utf8");
        },
        async read() {
            await this.ensureFile();
            return readFile(memoryPath, "utf8");
        },
        async appendDaily(entry) {
            await this.ensureFile();
            const lines = [`\n---\n\n## ${new Date().toISOString().slice(0, 10)}`, "", "- **Did:**", ...entry.did.map((item) => `  - ${item}`), "- **Decided:**", ...entry.decided.map((item) => `  - ${item}`), "- **Blocked:**", ...entry.blocked.map((item) => `  - ${item}`), `- **Next:** ${entry.next}`, ""];
            await appendFile(memoryPath, `${lines.join("\n")}\n`, "utf8");
            for (const item of [...entry.did, ...entry.decided, ...entry.blocked, entry.next])
                insertMemory.run(randomUUID(), projectRoot, "session", item, "MEMORY.md", 1, new Date().toISOString());
        },
        async remember(category, content) {
            await this.ensureFile();
            const line = `- **${category}:** ${content}`;
            await appendFile(memoryPath, `\n${line}\n`, "utf8");
            insertMemory.run(randomUUID(), projectRoot, category, content, "MEMORY.md", 1, new Date().toISOString());
        },
        async relevant(task, limit = 8) {
            const content = await this.read();
            const taskTerms = terms(task);
            const lines = content.split(/\r?\n/).filter((line) => line.trim().startsWith("-") || line.startsWith("## "));
            return lines.filter((line) => taskTerms.length === 0 || taskTerms.some((term) => line.toLowerCase().includes(term))).slice(-limit);
        },
        saveResume(state) {
            const updatedAt = new Date().toISOString();
            upsertResume.run({ projectRoot, task: state.task, sessionId: state.sessionId ?? null, currentChunk: state.currentChunk, status: state.status, nextStep: state.nextStep, updatedAt });
        },
        getResume() {
            const result = readResume.get(projectRoot);
            return result;
        },
        close() { db.close(); },
    };
}
//# sourceMappingURL=store.js.map