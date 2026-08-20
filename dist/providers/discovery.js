async function getJson(url, fetchImpl) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 700);
    try {
        const response = await fetchImpl(url, { signal: controller.signal });
        return response.ok ? await response.json() : undefined;
    }
    catch {
        return undefined;
    }
    finally {
        clearTimeout(timer);
    }
}
export async function discoverLocalRuntimes(fetchImpl = fetch) {
    const runtimes = [];
    const ollama = await getJson("http://127.0.0.1:11434/api/tags", fetchImpl);
    if (ollama)
        runtimes.push({ name: "ollama", baseUrl: "http://127.0.0.1:11434/v1", models: (ollama.models ?? []).map((model) => model.name ?? model.model ?? "").filter(Boolean), available: true, detail: "Ollama model catalog available" });
    const lmStudio = await getJson("http://127.0.0.1:1234/v1/models", fetchImpl);
    if (lmStudio)
        runtimes.push({ name: "lm-studio", baseUrl: "http://127.0.0.1:1234/v1", models: (lmStudio.data ?? []).map((model) => model.id ?? model.name ?? "").filter(Boolean), available: true, detail: "LM Studio OpenAI-compatible catalog available" });
    return runtimes;
}
//# sourceMappingURL=discovery.js.map