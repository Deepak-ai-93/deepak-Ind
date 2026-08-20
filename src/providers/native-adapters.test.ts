import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { AnthropicAdapter } from "./anthropic.js";
import { GoogleAdapter } from "./google.js";

describe("native provider adapters", () => {
  it("normalizes Anthropic message SSE events", async () => {
    const fetchImpl: typeof fetch = async () => new Response([
      'event: message_start', 'data: {"message":{"usage":{"input_tokens":5}}}', '',
      'event: content_block_delta', 'data: {"delta":{"text":"hello"}}', '',
      'event: message_delta', 'data: {"delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":2}}', '',
      'event: message_stop', 'data: {}', '',
    ].join("\n"), { status: 200, headers: { "content-type": "text/event-stream" } });
    const events = []; for await (const event of new AnthropicAdapter({ apiKey: "test", fetchImpl }).stream({ model: "claude-test", messages: [{ role: "user", content: "hi" }], maxOutputTokens: 8 })) events.push(event);
    assert.equal(events.some((event) => event.type === "delta" && event.text === "hello"), true);
    assert.equal(events.some((event) => event.type === "usage" && event.usage.totalTokens === 7), true);
  });

  it("normalizes Google streaming response chunks", async () => {
    const fetchImpl: typeof fetch = async () => new Response([
      'data: {"candidates":[{"content":{"parts":[{"text":"hello"}]}}],"usageMetadata":{"promptTokenCount":4,"candidatesTokenCount":3,"totalTokenCount":7}}', '',
    ].join("\n"), { status: 200, headers: { "content-type": "text/event-stream" } });
    const events = []; for await (const event of new GoogleAdapter({ apiKey: "test", fetchImpl }).stream({ model: "gemini-test", messages: [{ role: "user", content: "hi" }], maxOutputTokens: 8 })) events.push(event);
    assert.equal(events.some((event) => event.type === "delta" && event.text === "hello"), true);
    assert.equal(events.some((event) => event.type === "usage" && event.usage.totalTokens === 7), true);
  });
});
