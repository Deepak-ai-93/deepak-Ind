import { ProviderError, type ChatEvent, type ChatRequest, type ProviderAdapter, type ProviderCapabilities, type Usage } from "./types.js";

interface AnthropicOptions { apiKey: string; baseUrl?: string; fetchImpl?: typeof fetch; }

export class AnthropicAdapter implements ProviderAdapter {
  readonly id = "anthropic";
  private readonly apiKey: string;
  private readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: AnthropicOptions) {
    if (!options.apiKey) throw new Error("Anthropic requires ANTHROPIC_API_KEY.");
    this.apiKey = options.apiKey;
    this.baseUrl = (options.baseUrl ?? "https://api.anthropic.com").replace(/\/+$/, "");
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  capabilities(): ProviderCapabilities { return { streaming: true, tools: true, usage: true, cancellation: true, jsonMode: false }; }

  async *stream(request: ChatRequest): AsyncIterable<ChatEvent> {
    yield { type: "start", provider: this.id, model: request.model };
    const system = request.messages.filter((message) => message.role === "system").map((message) => message.content).join("\n\n");
    const messages = request.messages.filter((message) => message.role !== "system").map((message) => ({ role: message.role === "assistant" ? "assistant" : "user", content: message.content }));
    const headers = { "content-type": "application/json", "accept": "text/event-stream", "x-api-key": this.apiKey, "anthropic-version": "2023-06-01" };
    const body = { model: request.model, max_tokens: request.maxOutputTokens, messages, stream: true, ...(system ? { system } : {}) };
    let response: Response;
    try { response = await this.fetchImpl(`${this.baseUrl}/v1/messages`, { method: "POST", headers, body: JSON.stringify(body), ...(request.signal ? { signal: request.signal } : {}) }); }
    catch (error) { throw new ProviderError(this.id, `Request failed: ${error instanceof Error ? error.message : String(error)}`, { retryable: true }); }
    if (!response.ok) throw new ProviderError(this.id, `Provider returned HTTP ${response.status}: ${(await response.text()).slice(0, 300)}`, { status: response.status, retryable: response.status === 429 || response.status >= 500 });
    if (!response.body) throw new ProviderError(this.id, "Provider returned an empty response body.", { retryable: true });
    const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
    let buffer = ""; let eventName = ""; let finishReason: string | null = null; let usage: Usage | undefined;
    try {
      while (true) {
        const { value, done } = await reader.read(); if (done) break; buffer += value;
        const lines = buffer.split(/\r?\n/); buffer = lines.pop() ?? "";
        for (const line of lines) {
          if (line.startsWith("event:")) { eventName = line.slice(6).trim(); continue; }
          if (!line.startsWith("data:")) continue;
          let data: unknown; try { data = JSON.parse(line.slice(5).trim()) as unknown; } catch { continue; }
          if (!data || typeof data !== "object") continue;
          const record = data as Record<string, unknown>;
          if (eventName === "message_start" && record.message && typeof record.message === "object") {
            const startUsage = (record.message as Record<string, unknown>).usage as Record<string, unknown> | undefined;
            const input = startUsage?.input_tokens; if (typeof input === "number") usage = { inputTokens: input, outputTokens: 0, totalTokens: input, estimated: false };
          }
          if (eventName === "content_block_delta" && record.delta && typeof record.delta === "object") {
            const text = (record.delta as Record<string, unknown>).text; if (typeof text === "string") yield { type: "delta", text };
          }
          if (eventName === "message_delta") {
            const delta = record.delta as Record<string, unknown> | undefined; if (typeof delta?.stop_reason === "string") finishReason = delta.stop_reason;
            const output = (record.usage as Record<string, unknown> | undefined)?.output_tokens;
            if (usage && typeof output === "number") usage = { ...usage, outputTokens: output, totalTokens: usage.inputTokens + output };
            if (usage) yield { type: "usage", usage };
          }
          if (eventName === "message_stop") { yield { type: "done", finishReason }; return; }
        }
      }
      if (usage) yield { type: "usage", usage }; yield { type: "done", finishReason };
    } finally { reader.releaseLock(); }
  }
}
