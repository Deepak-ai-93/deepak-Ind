import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { inspectRepository } from "./repository.js";
import { selectContext } from "./selector.js";

describe("repository context selection", () => {
  it("ignores generated directories and ranks task-relevant files", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-context-"));
    await mkdir(join(root, "src"));
    await mkdir(join(root, "dist"));
    await writeFile(join(root, "src", "auth.ts"), "export function authenticate() { return true; }\n");
    await writeFile(join(root, "src", "billing.ts"), "export function charge() { return false; }\n");
    await writeFile(join(root, "dist", "generated.js"), "should not be selected\n");
    const snapshot = await inspectRepository(root);
    const selection = await selectContext(snapshot, "fix authentication", 100);
    assert.deepEqual(snapshot.files.map((file) => file.relativePath), ["src/auth.ts", "src/billing.ts"]);
    assert.equal(selection.selected[0]?.relativePath, "src/auth.ts");
    assert.ok(selection.estimatedTokens <= 100);
    assert.ok(snapshot.ignoredDirectories.includes("dist"));
  });

  it("never exceeds the context budget and reports omissions", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-budget-"));
    await writeFile(join(root, "large.ts"), "x".repeat(1_000));
    await writeFile(join(root, "small.ts"), "export const small = true;\n");
    const selection = await selectContext(await inspectRepository(root), "small", 10);
    assert.ok(selection.estimatedTokens <= 10);
    assert.ok(selection.omitted.length >= 1);
  });
});
