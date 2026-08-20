import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { openMemoryStore } from "./store.js";

describe("project memory", () => {
  it("creates append-only Markdown memory and reloads resume state", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-memory-"));
    const store = openMemoryStore(root);
    await store.ensureFile();
    await store.remember("decision", "Use a local-first provider adapter.");
    await store.appendDaily({ did: ["Completed the provider adapter"], decided: ["Keep task chunks approved"], blocked: [], next: "Build memory resume" });
    const markdown = await readFile(join(root, "MEMORY.md"), "utf8");
    assert.match(markdown, /local-first provider adapter/);
    assert.match(markdown, /Build memory resume/);
    store.saveResume({ task: "Build memory resume", currentChunk: 2, status: "active", nextStep: "Load memory before planning" });
    assert.deepEqual(store.getResume()?.currentChunk, 2);
    assert.ok((await store.relevant("provider"))[0]?.includes("provider"));
    store.close();
    const reopened = openMemoryStore(root);
    assert.equal(reopened.getResume()?.task, "Build memory resume");
    reopened.close();
  });
});
