import { ProviderError } from "./types.js";
function normalizeBaseUrl(baseUrl) {
    return baseUrl.replace(/\/+$/, "");
}
function toMessage(message) {
    return { role: message.role, content: message.content, ...(message.name ? { name: message.name } : {}), ...(message.toolCallId ? { tool_call_id: message.toolCallId } : {}) };
}
function toTools(tools) {
    return tools?.map((tool) => ({ type: "function", function: tool }));
}
function usageFrom(value) {
    if (!value || typeof value !== "object")
        return undefined;
    const usage = value;
    const input = typeof usage.prompt_tokens === "number" ? usage.prompt_tokens : undefined;
    const output = typeof usage.completion_tokens === "number" ? usage.completion_tokens : undefined;
    if (input === undefined || output === undefined)
        return undefined;
    const details = usage.prompt_tokens_details;
    const cached = details && typeof details === "object" ? details.cached_tokens : undefined;
    return { inputTokens: input, outputTokens: output, totalTokens: typeof usage.total_tokens === "number" ? usage.total_tokens : input + output, ...(typeof cached === "number" ? { cachedTokens: cached } : {}), estimated: false };
}
function parseSseLine(line) {
    if (!line.startsWith("data:"))
        return undefined;
    const payload = line.slice(5).trim();
    if (payload === "[DONE]")
        return "[DONE]";
    if (!payload)
        return undefined;
    try {
        return JSON.parse(payload);
    }
    catch {
        return undefined;
    }
}
export class OpenAICompatibleAdapter {
    id;
    baseUrl;
    apiKey;
    fetchImpl;
    constructor(options) {
        if (!options.baseUrl.trim())
            throw new Error("OpenAI-compatible provider requires a base URL.");
        this.baseUrl = normalizeBaseUrl(options.baseUrl);
        this.apiKey = options.apiKey;
        this.id = options.id ?? "openai-compatible";
        this.fetchImpl = options.fetchImpl ?? fetch;
    }
    capabilities() {
        return { streaming: true, tools: true, usage: true, cancellation: true, jsonMode: true };
    }
    async *stream(request) {
        yield { type: "start", provider: this.id, model: request.model };
        const headers = { "content-type": "application/json", accept: "text/event-stream" };
        if (this.apiKey)
            headers.authorization = `Bearer ${this.apiKey}`;
        const body = { model: request.model, messages: request.messages.map(toMessage), max_tokens: request.maxOutputTokens, stream: true, stream_options: { include_usage: true }, ...(request.temperature === undefined ? {} : { temperature: request.temperature }), ...(request.tools ? { tools: toTools(request.tools) } : {}), ...(request.jsonMode ? { response_format: { type: "json_object" } } : {}) };
        let response;
        try {
            response = await this.fetchImpl(`${this.baseUrl}/chat/completions`, { method: "POST", headers, body: JSON.stringify(body), ...(request.signal ? { signal: request.signal } : {}) });
        }
        catch (error) {
            if (request.signal?.aborted)
                throw error;
            throw new ProviderError(this.id, `Request failed: ${error instanceof Error ? error.message : String(error)}`, { retryable: true });
        }
        if (!response.ok) {
            const detail = await response.text().catch(() => "");
            throw new ProviderError(this.id, `Provider returned HTTP ${response.status}${detail ? `: ${detail.slice(0, 300)}` : ""}`, { status: response.status, retryable: response.status === 408 || response.status === 429 || response.status >= 500 });
        }
        if (!response.body)
            throw new ProviderError(this.id, "Provider returned an empty response body.", { retryable: true });
        const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
        let buffer = "";
        let finishReason = null;
        try {
            while (true) {
                const { value, done } = await reader.read();
                if (done)
                    break;
                buffer += value;
                const lines = buffer.split(/\r?\n/);
                buffer = lines.pop() ?? "";
                for (const line of lines) {
                    const parsed = parseSseLine(line);
                    if (parsed === undefined)
                        continue;
                    if (parsed === "[DONE]") {
                        yield { type: "done", finishReason };
                        return;
                    }
                    if (!parsed || typeof parsed !== "object")
                        continue;
                    const chunk = parsed;
                    const usage = usageFrom(chunk.usage);
                    if (usage)
                        yield { type: "usage", usage };
                    const choices = Array.isArray(chunk.choices) ? chunk.choices : [];
                    const choice = choices[0];
                    if (!choice || typeof choice !== "object")
                        continue;
                    const record = choice;
                    if (typeof record.finish_reason === "string")
                        finishReason = record.finish_reason;
                    const delta = record.delta;
                    if (delta && typeof delta === "object") {
                        const text = delta.content;
                        if (typeof text === "string" && text)
                            yield { type: "delta", text };
                    }
                }
            }
            yield { type: "done", finishReason };
        }
        finally {
            reader.releaseLock();
        }
    }
}
//# sourceMappingURL=openai-compatible.js.map