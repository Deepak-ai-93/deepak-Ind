# Idea Brief — IND

## Positioning

IND is the transparent, token-efficient terminal coding agent for developers who want one workflow across local models and hosted providers.

## Research signals

1. Pi treats compaction as a core coding-agent mechanism: it summarizes older context when the context window approaches a reserve threshold and tracks files read/modified in the summary. Source: https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/compaction.md
2. Aider maps a codebase before coding, reinforcing that repository structure and file relevance are central to reducing unnecessary context. Source: https://aider.chat/docs/repomap.html
3. OpenCode supports 75+ providers and local models through provider configuration, which validates demand for a provider-neutral experience but also raises the bar for IND’s differentiation. Source: https://dev.opencode.ai/docs/providers
4. Ollama exposes an OpenAI-compatible endpoint with streaming, JSON mode, tools, and usage reporting support, making OpenAI-compatible local endpoints a practical first adapter target. Source: https://github.com/ollama/ollama/blob/main/docs/api/openai-compatibility.mdx
5. Recent agent-harness research points toward workload-specialized routing, lazy tool discovery, and adaptive context compaction as promising ways to reduce agent overhead. Source: https://arxiv.org/abs/2603.05344

## Competitor teardown

| Product / workaround | Strength | Gap IND can target |
|---|---|---|
| Pi | Minimal terminal harness, compaction, extensibility | IND can make savings, task chunks, and usage telemetry more explicit |
| Aider | Repository map and strong Git-oriented workflow | IND can unify local/cloud providers and expose a richer usage ledger |
| OpenCode | Broad provider catalog and local-model support | IND can differentiate around measurable context budgets and chunk checkpoints |
| DIY scripts | Maximum control and privacy | High setup cost; no consistent memory, routing, or usage accounting |

## TAM / SAM / SOM framing

- TAM: developers using AI coding assistants and terminal workflows.
- SAM: developers who prefer CLI tools and care about cost, privacy, model choice, or reproducibility.
- SOM: early adopters in open-source AI, local-LLM, terminal, and developer-tool communities reachable through GitHub and npm.

## Channels

- GitHub repository with benchmark results and provider adapters.
- npm distribution and cross-platform install docs.
- Demo videos showing identical tasks with baseline vs IND token ledger.
- Communities around local models, terminal tooling, and AI coding agents.

## Pricing / business hypothesis

Keep the MVP free and local-first. Later options: paid hosted sync, team usage dashboards, or enterprise policy controls. Do not add billing to v1.

## Differentiation thesis

IND should not promise merely “a smarter agent.” It should show the user exactly why each token was sent, which context was omitted, how much was saved against a baseline, and when a task chunk is safe to continue.

## Validation moves

- Run a fixed 10-task benchmark across IND and a full-context baseline.
- Interview or observe five developers who already use terminal coding agents.
- Publish one reproducible report showing token savings, completion rate, latency, and failure modes across one local and two hosted providers.

