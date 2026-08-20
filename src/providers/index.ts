import { loadConfig } from "../config.js";
import { AnthropicAdapter } from "./anthropic.js";
import { discoverLocalRuntimes } from "./discovery.js";
import { GoogleAdapter } from "./google.js";
import { OpenAICompatibleAdapter } from "./openai-compatible.js";
import type { ProviderAdapter } from "./types.js";
import { assertProviderAllowed, loadPolicy } from "../policy/policy.js";

export function createConfiguredProvider(): ProviderAdapter {
  const config = loadConfig();
  assertProviderAllowed(config.provider, loadPolicy(config.projectRoot));
  if (config.provider === "anthropic") return new AnthropicAdapter({ apiKey: process.env.ANTHROPIC_API_KEY ?? "" });
  if (config.provider === "google") return new GoogleAdapter({ apiKey: process.env.GOOGLE_GENERATIVE_AI_API_KEY ?? "" });
  if (config.provider === "openai") return new OpenAICompatibleAdapter({ id: "openai", baseUrl: config.baseUrl ?? "https://api.openai.com/v1", apiKey: process.env.OPENAI_API_KEY });
  return new OpenAICompatibleAdapter({ baseUrl: config.baseUrl ?? "http://localhost:11434/v1", apiKey: process.env.OPENAI_API_KEY });
}

export function providerSummary(): string[] {
  const config = loadConfig();
  assertProviderAllowed(config.provider, loadPolicy(config.projectRoot));
  const capabilities = createConfiguredProvider().capabilities();
  return [`configured: ${config.provider}`, `model: ${config.model || "not selected"}`, `endpoint: ${config.baseUrl ?? "provider default"}`, `capabilities: ${Object.entries(capabilities).filter(([, supported]) => supported).map(([name]) => name).join(", ")}`];
}

export async function localRuntimeSummary(): Promise<string[]> {
  const runtimes = await discoverLocalRuntimes();
  if (runtimes.length === 0) return ["no local runtimes detected (probes: Ollama 11434, LM Studio 1234)"];
  return runtimes.flatMap((runtime) => [`${runtime.name}: ${runtime.baseUrl}`, `models: ${runtime.models.join(", ") || "none reported"}`]);
}



