import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { scanProjectSecurity } from "./scanner.js";

describe("security scanner", () => {
  it("flags likely secrets but ignores the example environment file", async () => {
    const root = await mkdtemp(join(tmpdir(), "ind-security-"));
    await writeFile(join(root, "safe.ts"), "const key = process.env.OPENAI_API_KEY;\n");
    const fakeSecret = ["sk-", "12345678901234567890"].join("");
    await writeFile(join(root, "unsafe.ts"), `const key = '${fakeSecret}';\n`);
    await writeFile(join(root, ".env.example"), "OPENAI_API_KEY=\n");
    const issues = await scanProjectSecurity(root);
    assert.equal(issues.length, 1);
    assert.equal(issues[0]?.file, "unsafe.ts");
  });
});
