# TODO — IND (Rust Port & Migration)

> **Goal**: Complete port of IND from TypeScript/Node.js to high-performance native Rust for standalone, zero-dependency, ultra-low memory execution across Windows, macOS, and Linux.

## P0 — Core Foundation & Execution Engine
- [x] (P0) #101 Setup `Cargo.toml` with dependencies (`clap`, `tokio`, `serde`, `serde_json`, `reqwest`, `rusqlite`, `ignore`, `colored`, etc.).
- [x] (P0) #102 Port configuration loading (`src-rust/config/mod.rs`) for `.ind/config.json`, env vars, and project root discovery.
- [x] (P0) #103 Port CLI commands & argument parser (`src-rust/main.rs`) using `clap` (`run`, `plan`, `context`, `route`, `budget`, `providers`, `discover`, `usage`, `memory`, `resume`, `benchmark`, `leaderboard`, `doctor`, `config`, `policy`).
- [x] (P0) #104 Port Context Indexer & Token Budget Selection (`src-rust/context/`) with `ignore`-based traversal, TS-parity scoring, and token-budgeted selection.
- [x] (P0) #105 Port Provider Engine & Adapters (`src-rust/providers/`) with streaming SSE adapters for OpenAI-compatible, Anthropic, Google, plus local runtime discovery for Ollama / LM Studio.
- [x] (P0) #106 Port Tool Execution & File Editing Engine (`src-rust/tools/`) with safe project paths, expected-content edit preconditions, policy checks, destructive-command blocks, and timeouts.
- [x] (P0) #107 Port Task Planner & Approval Workflow (`src-rust/tasks/`) with 3-chunk plans and chunked execution (`ind plan`, `ind run`).

## P1 — Memory, Usage & Telemetry
- [x] (P1) #108 Port Local Project Memory (`src-rust/memory/`) with append-only `MEMORY.md`, JSON-backed entries, and resume state. *(AES-256-GCM encrypted sync still stubbed in `memory sync`.)*
- [x] (P1) #109 Port SQLite Usage Ledger and token cost tracking (`src-rust/usage/`, `src-rust/budget/`) with budget planning, cost/savings estimation. *(Live terminal monitor pending.)*
- [x] (P1) #110 Port Resumable Sessions & State Checkpoints (`src-rust/memory/` resume state, `ind resume`). *(`src-rust/replay/` budget-envelope replay pending.)*
- [ ] (P1) #111 Port Diagnostics and Security Scanner (`ind doctor` renders diagnostics; `src-rust/security/` scanner pending).

## P2 — Extended Features & Benchmarking
- [x] (P2) #112 Port Local Runtime Discovery for Ollama / LM Studio (`src-rust/providers/discovery.rs`, `ind discover`).
- [ ] (P2) #113 Port Benchmarking suite & Leaderboard generation (`src-rust/benchmark/`, `ind leaderboard` writes reports).
- [ ] (P2) #114 Setup automated GitHub Actions CI for cross-platform binary releases (Windows `.exe`, Linux `x86_64`/`aarch64`, macOS universal).

## Build Status
- `cargo build` — clean, no warnings
- `cargo test` — 33 tests passing (context selection, routing, budget, planner, runner, tools, memory, policy, provider parsing)