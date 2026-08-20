import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { appendJsonlEvent, readJsonlEvents } from "./jsonl.js";

describe("JSONL integration", () => {
  it("writes events, redacts secrets, and reads them back", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-jsonl-"));
    const path = join(root, "events.jsonl");
    await appendJsonlEvent(path, { type: "provider.call", model: "test", apiKey: "sk-12345678901234567890", message: "token=secret-value" });
    const events = await readJsonlEvents(path);
    assert.equal(events.length, 1);
    assert.equal(events[0]?.apiKey, "[REDACTED]");
    assert.match(await readFile(path, "utf8"), /REDACTED/);
  });
});
