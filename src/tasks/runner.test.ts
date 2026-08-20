import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createTaskPlan } from "./planner.js";
import { runTaskPlan } from "./runner.js";

describe("task planning and execution", () => {
  it("runs approved chunks, applies an edit, and verifies a command", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-runner-"));
    const plan = createTaskPlan("add a marker file");
    const events: string[] = [];
    const result = await runTaskPlan(plan, new Map([
      [plan.chunks[0]!.id, []],
      [plan.chunks[1]!.id, [{ type: "edit", path: "marker.txt", content: "IND\n" }]],
      [plan.chunks[2]!.id, [{ type: "command", command: process.platform === "win32" ? "node -e \"process.exit(0)\"" : "node -e 'process.exit(0)'" }]],
    ]), { projectRoot: root, approvedChunks: new Set(plan.chunks.map((chunk) => chunk.id)) }, (event) => events.push(event.type));
    assert.equal(result.chunks.every((chunk) => chunk.status === "passed"), true);
    assert.equal(await readFile(join(root, "marker.txt"), "utf8"), "IND\n");
    assert.deepEqual(events.filter((event) => event === "approval-required").length, 3);
    assert.equal(events.at(-1), "plan-complete");
  });

  it("blocks before running an unapproved chunk", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-approval-"));
    const plan = createTaskPlan("do not run");
    await assert.rejects(() => runTaskPlan(plan, new Map(), { projectRoot: root, approvedChunks: new Set() }), /blocked until approved/);
    assert.equal(plan.chunks[0]?.status, "blocked");
  });
});
