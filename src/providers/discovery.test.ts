import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { discoverLocalRuntimes } from "./discovery.js";

describe("local runtime discovery", () => {
  it("detects Ollama and LM Studio without sending project content", async () => {
    const calls: string[] = [];
    const fakeFetch: typeof fetch = async (input) => {
      const url = String(input); calls.push(url);
      if (url.includes("11434")) return new Response(JSON.stringify({ models: [{ name: "llama3.2" }] }), { status: 200 });
      return new Response(JSON.stringify({ data: [{ id: "local-coder" }] }), { status: 200 });
    };
    const runtimes = await discoverLocalRuntimes(fakeFetch);
    assert.deepEqual(runtimes.map((runtime) => runtime.name), ["ollama", "lm-studio"]);
    assert.deepEqual(runtimes[0]?.models, ["llama3.2"]);
    assert.deepEqual(runtimes[1]?.models, ["local-coder"]);
    assert.ok(calls.every((url) => url.startsWith("http://127.0.0.1")));
  });
});
