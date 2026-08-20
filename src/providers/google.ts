import { ProviderError, type ChatEvent, type ChatRequest, type ProviderAdapter, type ProviderCapabilities, type Usage } from "./types.js";

interface GoogleOptions { apiKey: string; baseUrl?: string; fetchImpl?: typeof fetch; }

export class GoogleAdapter implements ProviderAdapter {
  readonly id = "google";
  private readonly apiKey: string; private readonly baseUrl: string; private readonly fetchImpl: typeof fetch;
  constructor(options: GoogleOptions) { if (!options.apiKey) throw new Error("Google requires GOOGLE_GENERATIVE_AI_API_KEY."); this.apiKey = options.apiKey; this.baseUrl = (options.baseUrl ?? "https://generativelanguage.googleapis.com").replace(/\/+$/, ""); this.fetchImpl = options.fetchImpl ?? fetch; }
  capabilities(): ProviderCapabilities { return { streaming: true, tools: true, usage: true, cancellation: true, jsonMode: true }; }
  async *stream(request: ChatRequest): AsyncIterable<ChatEvent> {
    yield { type: "start", provider: this.id, model: request.model };
    const system = request.messages.filter((message) => message.role === "system").map((message) => message.content).join("\n\n");
    const contents = request.messages.filter((message) => message.role !== "system").map((message) => ({ role: message.role === "assistant" ? "model" : "user", parts: [{ text: message.content }] }));
    const body = { contents, ...(system ? { systemInstruction: { parts: [{ text: system }] } } : {}), generationConfig: { maxOutputTokens: request.maxOutputTokens, ...(request.temperature === undefined ? {} : { temperature: request.temperature }) } };
    const response = await this.fetchImpl(`${this.baseUrl}/v1beta/models/${encodeURIComponent(request.model)}:streamGenerateContent?alt=sse`, { method: "POST", headers: { "content-type": "application/json", "x-goog-api-key": this.apiKey }, body: JSON.stringify(body), ...(request.signal ? { signal: request.signal } : {}) });
    if (!response.ok) throw new ProviderError(this.id, `Provider returned HTTP ${response.status}: ${(await response.text()).slice(0, 300)}`, { status: response.status, retryable: response.status === 429 || response.status >= 500 });
    if (!response.body) throw new ProviderError(this.id, "Provider returned an empty response body.", { retryable: true });
    const reader = response.body.pipeThrough(new TextDecoderStream()).getReader(); let buffer = ""; let usage: Usage | undefined;
    try {
      while (true) { const { value, done } = await reader.read(); if (done) break; buffer += value; const lines = buffer.split(/\r?\n/); buffer = lines.pop() ?? "";
        for (const line of lines) { if (!line.startsWith("data:")) continue; let data: unknown; try { data = JSON.parse(line.slice(5).trim()) as unknown; } catch { continue; } if (!data || typeof data !== "object") continue;
          const record = data as Record<string, unknown>; const first = (Array.isArray(record.candidates) ? record.candidates[0] : undefined) as Record<string, unknown> | undefined; const content = first?.content as Record<string, unknown> | undefined; const parts = Array.isArray(content?.parts) ? content.parts : []; const text = (parts[0] as Record<string, unknown> | undefined)?.text; if (typeof text === "string") yield { type: "delta", text };
          const metadata = record.usageMetadata as Record<string, unknown> | undefined; if (typeof metadata?.promptTokenCount === "number" && typeof metadata.candidatesTokenCount === "number") usage = { inputTokens: metadata.promptTokenCount, outputTokens: metadata.candidatesTokenCount, totalTokens: typeof metadata.totalTokenCount === "number" ? metadata.totalTokenCount : metadata.promptTokenCount + metadata.candidatesTokenCount, estimated: false };
        }
      }
      if (usage) yield { type: "usage", usage }; yield { type: "done", finishReason: null };
    } finally { reader.releaseLock(); }
  }
}
