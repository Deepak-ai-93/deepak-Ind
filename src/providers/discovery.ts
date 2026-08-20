export interface LocalRuntime {
  name: "ollama" | "lm-studio" | "openai-compatible";
  baseUrl: string;
  models: string[];
  available: boolean;
  detail: string;
}

interface ModelResponse { data?: Array<{ id?: string; name?: string }> }

async function getJson(url: string, fetchImpl: typeof fetch): Promise<unknown> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 700);
  try { const response = await fetchImpl(url, { signal: controller.signal }); return response.ok ? await response.json() as unknown : undefined; } catch { return undefined; } finally { clearTimeout(timer); }
}

export async function discoverLocalRuntimes(fetchImpl: typeof fetch = fetch): Promise<LocalRuntime[]> {
  const runtimes: LocalRuntime[] = [];
  const ollama = await getJson("http://127.0.0.1:11434/api/tags", fetchImpl) as { models?: Array<{ name?: string; model?: string }> } | undefined;
  if (ollama) runtimes.push({ name: "ollama", baseUrl: "http://127.0.0.1:11434/v1", models: (ollama.models ?? []).map((model) => model.name ?? model.model ?? "").filter(Boolean), available: true, detail: "Ollama model catalog available" });
  const lmStudio = await getJson("http://127.0.0.1:1234/v1/models", fetchImpl) as ModelResponse | undefined;
  if (lmStudio) runtimes.push({ name: "lm-studio", baseUrl: "http://127.0.0.1:1234/v1", models: (lmStudio.data ?? []).map((model) => model.id ?? model.name ?? "").filter(Boolean), available: true, detail: "LM Studio OpenAI-compatible catalog available" });
  return runtimes;
}
