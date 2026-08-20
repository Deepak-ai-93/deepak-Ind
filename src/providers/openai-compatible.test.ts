import { createServer, type Server } from "node:http";
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { OpenAICompatibleAdapter } from "./openai-compatible.js";
import type { ChatEvent } from "./types.js";

let server: Server;
let baseUrl = "";

before(async () => {
  server = createServer((request, response) => {
    assert.equal(request.method, "POST");
    assert.equal(request.url, "/v1/chat/completions");
    response.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-cache" });
    response.write('data: {"choices":[{"delta":{"content":"hello "}}]}\n\n');
    response.write('data: {"choices":[{"delta":{"content":"IND"},"finish_reason":"stop"}]}\n\n');
    response.write('data: {"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":4,"total_tokens":16}}\n\n');
    response.end("data: [DONE]\n\n");
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address === "object");
  baseUrl = `http://127.0.0.1:${address.port}/v1`;
});

after(async () => {
  await new Promise<void>((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
});

describe("OpenAI-compatible provider", () => {
  it("streams text and normalizes usage", async () => {
    const provider = new OpenAICompatibleAdapter({ baseUrl, apiKey: "test-key" });
    const events: ChatEvent[] = [];
    for await (const event of provider.stream({ model: "test-model", messages: [{ role: "user", content: "hello" }], maxOutputTokens: 32 })) events.push(event);
    assert.deepEqual(events.filter((event) => event.type === "delta").map((event) => event.type === "delta" ? event.text : ""), ["hello ", "IND"]);
    const usage = events.find((event) => event.type === "usage");
    assert.deepEqual(usage, { type: "usage", usage: { inputTokens: 12, outputTokens: 4, totalTokens: 16, estimated: false } });
    assert.deepEqual(events.at(-1), { type: "done", finishReason: "stop" });
  });

  it("exposes a capability contract", () => {
    const provider = new OpenAICompatibleAdapter({ baseUrl });
    assert.deepEqual(provider.capabilities(), { streaming: true, tools: true, usage: true, cancellation: true, jsonMode: true });
  });
});
