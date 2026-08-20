# Stack Blueprint — IND

> Assembled by `pack-builder.mjs` from `pack-plan.json` · 2026-08-20. This file + `PRD.md` + `sitemap.md` are **the build pack**: everything the builder needs, nothing it doesn't. Tool-agnostic — works in any CLI (Claude Code, Cursor, Codex), Lovable, Bolt, v0.

## 1. Identity & verdict

| Field                      | Value                                                                                                                                                                                                                     |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **App name / one-liner**   | IND — A cross-platform terminal coding agent that saves tokens, chunks work, shows usage, and remembers the project.                                                                                                      |
| **Idea given by user**     | I want to build my own terminal coding tool like PI, named IND, with less AI token usage, chunked tasks, usage monitoring in the terminal, and memory. It should work on all devices and support local and all providers. |
| **Stack preference given** | No fixed stack; choose a portable TypeScript and Node.js CLI architecture.                                                                                                                                                |
| **Evaluation verdict**     | 26/35 → ITERATE (computed by `saas-score.mjs`)                                                                                                                                                                            |
| **Kill guardrail**         | Within 30 days of public testing, produce 10 benchmark runs across at least three environments and demonstrate median input-token reduction of at least 20 percent without reducing verified task completion.             |
| **Audience**               | Individual developers and small engineering teams using terminal-based AI coding tools.                                                                                                                                   |
| **Monetized?**             | no                                                                                                                                                                                                                        |
| **Mode**                   | new project                                                                                                                                                                                                               |

## 2. Stack lock (NO more decisions after this)

| Layer     | Locked choice                                                                     | Version/notes                                                                                                                                                                     |
| --------- | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Framework | Node.js 20+ CLI with TypeScript and a thin command dispatcher                     | Use provider adapters, capability detection, AbortController cancellation, zod schemas, Vitest, and fixture repositories. Keep the core independent from any single provider SDK. |
| UI        | Ink-based terminal UI with ANSI-safe fallback output                              |                                                                                                                                                                                   |
| Fonts     | Terminal default; no web fonts                                                    |                                                                                                                                                                                   |
| Data      | SQLite via better-sqlite3 or a portable equivalent plus Markdown memory files     |                                                                                                                                                                                   |
| Auth      | No account auth in MVP; environment variables and local credential storage        |                                                                                                                                                                                   |
| Payments  | None in MVP                                                                       | — (not monetized)                                                                                                                                                                 |
| Hosting   | npm package and platform-specific release artifacts                               | → deploy per deploy-runbook.md (host: npm and GitHub Releases)                                                                                                                    |
| Analytics | Local SQLite usage ledger and opt-in JSONL export; no remote analytics by default |                                                                                                                                                                                   |

> If the user gave a different stack preference, honor it — but lock it here exactly the same way.

## 3. Design — source of truth + design system (applied as-is)

- **Design source of truth (locked, never "TBD"):** **Open-source design pack** (`templates/design-system.md`) — applied as-is.
- **Palette:** neutral shadcn tokens + one accent (`--primary` hue only): `174 72% 45%`
- **Notes:** Dark terminal-first interface with teal accent, semantic green/yellow/red states, compact status bar, explicit token ledger, no decorative panels. Must remain usable without color.
- **Design parity:** every screen is visually checked against the source of truth (browser-MCP screenshot vs Figma/Stitch) at 375/768/1280 — `frontend-design.md` §3/§5.
- **Pages & components map (summary — full blocks in `sitemap.md` §2):**

| Page/Route     | Components (shadcn)                                                                                         | Notes                                                            |
| -------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| /ind           | command prompt, task plan, chunk list, diff preview, approval prompt, tool output, token ledger, status bar | Enter a task and execute it through visible chunks.              |
| /ind usage     | date/model filters, session table, chunk table, totals, savings comparison, export action                   | Explain where tokens and time were spent and what IND saved.     |
| /ind memory    | memory sections, recent decisions, open questions, file references, edit/export/reset commands              | Make persistent context inspectable and maintainable.            |
| /ind benchmark | fixture selector, provider matrix, baseline toggle, progress, results table, report path                    | Measure token savings and completion quality against a baseline. |

## 4. Backend architecture (open-source, applied as-is)

- **Reference:** `templates/backend-architecture.md` — folder structure, auth flow, payments flow, security, ops.
- **Folder structure:** `src/cli, src/core, src/providers, src/context, src/memory, src/usage, src/tools, src/terminal, src/benchmark, src/storage`
- **Server actions:** `src/actions/{feature}.ts` — zod schema + ownership check + `revalidatePath`.
- **Auth:** `lib/auth.ts` + `middleware.ts` guarding `/dashboard/:path*` and `/api/:path*`.

### 4.1 AI features (locked rails)

- **Reference:** `templates/ai-logic.md` — streaming UX, prompts-as-code, cost rails, evals, security.
- **Repository context selection:** Selects the smallest relevant file and symbol set for a task. — model: cheap local or fast hosted model with deterministic heuristics first · streaming: no · cost rail: Hard input and output caps; prefer indexed metadata and cached summaries over raw file contents. · evals: 10 fixture tasks with expected relevant files and token ceilings.
- **Task planning and chunk execution:** Turns a request into bounded, reviewable coding chunks and produces edits plus verification steps. — model: cheap model for plan normalization; stronger configured model for hard edits · streaming: yes · cost rail: Per-chunk budget, maximum tool turns, timeout, cancellation, and model routing policy. · evals: Golden repository tasks covering add, modify, refactor, test, and bug-fix flows.
- **Memory summarization:** Compresses completed work and decisions into portable project memory. — model: cheap model with a strict schema and deterministic fallback · streaming: no · cost rail: Summarize only changed files and unresolved decisions; never resummarize unchanged memory. · evals: Continuation tests where a new session must recover decisions and next steps.
- **Prompts as code:** `lib/ai/prompts/{feature}.ts` with zod schemas + versioning — no literals in components.
- **Env vars:** AI keys server-only — never `NEXT_PUBLIC_`.

## 5. Data model (paste-ready)

| Entity         | Key fields                                                                                                                                | Relations                                                                       |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| projects       | id, root_path, display_name, created_at, updated_at                                                                                       | one project has many sessions and memory entries                                |
| sessions       | id, project_id, provider, model, started_at, ended_at, status, input_tokens, output_tokens, estimated_cost                                | many sessions belong to one project                                             |
| chunks         | id, session_id, sequence, title, goal, status, input_tokens, output_tokens, started_at, completed_at                                      | many chunks belong to one session                                               |
| usage_events   | id, session_id, chunk_id, provider, model, event_type, input_tokens, output_tokens, cached_tokens, latency_ms, estimated_cost, created_at | many usage events belong to a session and optional chunk                        |
| memory_entries | id, project_id, category, key, content, source, importance, created_at, updated_at                                                        | many memory entries belong to one project and mirror selected Markdown sections |

Rules: `user_id` FK on every owned table · `created_at` default now() · indexes on `user_id`, `status`, `email`.

**Env vars (paste into `.env.example`):**

| Var                          | Example                   | Where it comes from                 |
| ---------------------------- | ------------------------- | ----------------------------------- |
| OPENAI_API_KEY               | set in user environment   | OpenAI                              |
| ANTHROPIC_API_KEY            | set in user environment   | Anthropic                           |
| GOOGLE_GENERATIVE_AI_API_KEY | set in user environment   | Google                              |
| IND_BASE_URL                 | http://localhost:11434/v1 | Local or OpenAI-compatible endpoint |
| IND_MODEL                    | local-model               | User configuration                  |

## 6. Build order (the distraction-free sequence — do NOT skip ahead)

1. **Scaffold the cross-platform TypeScript CLI and configuration system** — done when: The ind command launches on Windows, macOS, and Linux fixtures, prints help, and loads safe configuration.
2. **Implement provider adapter contract and OpenAI-compatible endpoint** — done when: A streaming chat call, cancellation, capability detection, and normalized usage event work against a test server.
3. **Build repository inspection and token-efficient context selection** — done when: Fixture tasks select relevant files, cache summaries, truncate output, and stay under configured budgets.
4. **Implement chunked task planning, approval, tools, edits, and verification** — done when: A fixture coding task runs through visible chunks, applies a diff, executes an approved test, and records the result.
5. **Add SQLite usage ledger and live terminal monitor** — done when: The terminal shows per-call and aggregate tokens, cost estimates, latency, provider, model, and baseline savings.
6. **Add Markdown plus SQLite project memory and resume** — done when: A new session reloads decisions, changed files, open questions, and the next chunk without replaying the full transcript.
7. **Add native provider adapters and local runtime discovery** — done when: OpenAI, Anthropic, Google, and at least one local OpenAI-compatible runtime pass the common adapter contract tests.
8. **Create benchmark fixtures and baseline comparison** — done when: ind benchmark produces reproducible JSONL and Markdown results for token reduction, completion, latency, and failures.
9. **Harden cross-platform packaging, security, tests, and documentation** — done when: Install, permissions, secret handling, cancellation, errors, and release artifacts pass the production audit.
10. **Add advanced routing and optional JSONL integrations** — done when: Task-type model routing and opt-in event export work without changing the core workflow.

**Definition of done per step:** the app runs (`npm run dev`), the step's flow works end-to-end (and matches the design for UI steps), committed. Never 2 steps before running.

## 7. Handoff prompts — paste the pack into ANY builder

### A. CLI agent (Claude Code / Cursor / Codex / …)

```
Build the app described in PRD.md, stack-blueprint.md and sitemap.md in this folder. The sitemap is the map: every route/page/endpoint in it must exist, nothing else. Follow the build order exactly; keep the app runnable after every step; commit after each working feature; cover auth + billing with tests. Don't redesign — apply the locked design system and architecture as-is.
```

### B. Lovable / Bolt / v0 (web builders — paste everything)

```
Build a production-ready web app: A cross-platform terminal coding agent that saves tokens, chunks work, shows usage, and remembers the project..

STACK: Node.js 20+ CLI with TypeScript and a thin command dispatcher + Ink-based terminal UI with ANSI-safe fallback output + SQLite via better-sqlite3 or a portable equivalent plus Markdown memory files + No account auth in MVP; environment variables and local credential storage + None in MVP + npm package and platform-specific release artifacts.

PAGES: /ind (Enter a task and execute it through visible chunks.) · /ind usage (Explain where tokens and time were spent and what IND saved.) · /ind memory (Make persistent context inspectable and maintainable.) · /ind benchmark (Measure token savings and completion quality against a baseline.).
DESIGN: neutral shadcn tokens, accent 174 72% 45%, Terminal default; no web fonts fonts, command prompt, task plan, chunk list, diff preview, approval prompt, tool output, token ledger, status bar, date/model filters, session table, chunk table, totals, savings comparison, export action, memory sections, recent decisions, open questions, file references, edit/export/reset commands, fixture selector, provider matrix, baseline toggle, progress, results table, report path.
DATA MODEL: projects (id, root_path, display_name, created_at, updated_at) · sessions (id, project_id, provider, model, started_at, ended_at, status, input_tokens, output_tokens, estimated_cost) · chunks (id, session_id, sequence, title, goal, status, input_tokens, output_tokens, started_at, completed_at) · usage_events (id, session_id, chunk_id, provider, model, event_type, input_tokens, output_tokens, cached_tokens, latency_ms, estimated_cost, created_at) · memory_entries (id, project_id, category, key, content, source, importance, created_at, updated_at).
AUTH: No IND account auth in MVP; provider credentials stay in environment variables or an OS-appropriate local credential store.; protect the app area.
PAYMENTS: none in MVP.
ENV VARS (create .env.example): OPENAI_API_KEY (set in user environment) · ANTHROPIC_API_KEY (set in user environment) · GOOGLE_GENERATIVE_AI_API_KEY (set in user environment) · IND_BASE_URL (http://localhost:11434/v1) · IND_MODEL (local-model).
FEATURES (MVP, in order): Start an IND session in any supported terminal and detect the project root. → Inspect the repository and build a relevance-ranked context set instead of sending the whole tree. → Plan a request into bounded task chunks with a visible checkpoint before each execution chunk. → Execute file edits and verification commands through a safe tool loop with approval settings. → Support a common provider adapter contract for OpenAI-compatible endpoints plus native adapters for OpenAI, Anthropic, and Google. → Apply context compression, output truncation, prompt caching where available, and configurable token budgets. → Show live input tokens, output tokens, estimated cost, latency, provider, model, savings, and chunk status in the terminal. → Persist project memory as Markdown and structured session usage as SQLite, then reload it on the next session. → Provide a deterministic benchmark command comparing IND context usage to a full-context baseline..
QUALITY: mobile responsive, empty/loading/error states, accessibility, SEO meta. First build the landing page, then auth, then Start an IND session in any supported terminal and detect the project root., Inspect the repository and build a relevance-ranked context set instead of sending the whole tree., Plan a request into bounded task chunks with a visible checkpoint before each execution chunk.. Run/verify after every step. No gold-plating.
```

### C. Any tool, re-prompt after edits

```
Keep this project's design system and architecture unchanged. Implement the next item from PRD.md's must-haves exactly as scoped; run it; verify; commit.
```

## 8. Definition of done (before this pack is "ready")

- [ ] Every field in §1–§6 filled; stack locked; no open decisions left
- [ ] **Design source of truth locked** (pack) — not "TBD"; tokens mapped per `frontend-design.md`
- [ ] Data model SQL paste-ready; build order numbered and complete
- [ ] **AI section present** (PRD has AI features) — rails locked per `ai-logic.md`
- [ ] Handoff prompts filled in with the real app details
- [ ] **Deploy plan locked**: ONE host — npm and GitHub Releases — mirrored in `deploy-runbook.md` (generated by `deploy-setup.mjs` at Stage 7)
- [ ] PRD.md must-haves match the build order 1:1
- [ ] `validation.md` verdict recorded — ITERATE (26/35)
- [ ] Nothing in the pack references a tool-specific feature (works in CLI + web builders)