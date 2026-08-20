# Creator Portfolio — IND

## Identity

| Field | Value |
|---|---|
| Creator / team | IND creator |
| What you build | Developer tools and AI-assisted coding workflows |
| One-line taste | Fast, transparent, terminal-first, and economical with tokens |
| Preferred stack family | TypeScript + Node.js, cross-platform CLI |
| Time budget per app | MVP first; production hardening after the core loop works |

## Stack taste

| Layer | Default I like | Why / notes |
|---|---|---|
| Runtime | Node.js 20+ | Broad cross-platform support |
| Language | TypeScript | Safer provider and tool adapters |
| Terminal UI | Ink or a small TUI abstraction | Live usage panels without requiring a browser |
| Persistence | SQLite + human-readable Markdown | Structured metrics plus portable memory |
| Providers | Adapter interface for cloud and local models | Avoid vendor lock-in |
| Packaging | npm package with platform-neutral binaries/scripts | Installable on Windows, macOS, and Linux |

## Design taste

- Palette: dark terminal baseline with one clear accent and semantic status colors.
- Mood: dense but calm; every token, command, and tool action is explainable.
- Pet rules: keyboard-first, readable without color, no hidden background work, safe defaults.
- Avoid: opaque token usage, provider-specific behavior leaking into the core, and giant context dumps.

## Monetization patterns I accept

- No monetization decision for the MVP.

## Pet decisions

- IND must support local models and multiple hosted providers through one interface.
- Usage and cost must be visible in the terminal itself.
- Project memory must remain portable and inspectable.
