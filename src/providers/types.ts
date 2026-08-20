export type ChatRole = "system" | "user" | "assistant" | "tool";

export interface ChatMessage {
  role: ChatRole;
  content: string;
  name?: string;
  toolCallId?: string;
}

export interface ProviderCapabilities {
  streaming: boolean;
  tools: boolean;
  usage: boolean;
  cancellation: boolean;
  jsonMode: boolean;
}

export interface Usage {
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  cachedTokens?: number;
  estimated: boolean;
}

export interface ChatTool {
  name: string;
  description?: string;
  inputSchema: Record<string, unknown>;
}

export interface ChatRequest {
  model: string;
  messages: ChatMessage[];
  maxOutputTokens: number;
  temperature?: number;
  tools?: ChatTool[];
  jsonMode?: boolean;
  signal?: AbortSignal;
}

export type ChatEvent =
  | { type: "start"; provider: string; model: string }
  | { type: "delta"; text: string }
  | { type: "usage"; usage: Usage }
  | { type: "done"; finishReason: string | null }
  | { type: "error"; error: Error };

export interface ProviderAdapter {
  readonly id: string;
  capabilities(): ProviderCapabilities;
  stream(request: ChatRequest): AsyncIterable<ChatEvent>;
}

export class ProviderError extends Error {
  readonly status: number | undefined;
  readonly provider: string;
  readonly retryable: boolean;

  constructor(provider: string, message: string, options: { status?: number; retryable?: boolean } = {}) {
    super(message);
    this.name = "ProviderError";
    this.provider = provider;
    this.status = options.status;
    this.retryable = options.retryable ?? false;
  }
}
