# PRD — IND

> Assembled by `pack-builder.mjs` from `pack-plan.json` · 2026-08-20 · **approval contract — nothing is coded until the user approves it.**

## 1. Identity

| Field          | Value                                                                                                          |
| -------------- | -------------------------------------------------------------------------------------------------------------- |
| **App name**   | IND                                                                                                            |
| **One-liner**  | A cross-platform terminal coding agent that saves tokens, chunks work, shows usage, and remembers the project. |
| **Audience**   | Individual developers and small engineering teams using terminal-based AI coding tools.                        |
| **Platform**   | Cross-platform terminal CLI for Windows, macOS, and Linux                                                      |
| **Mode**       | new project                                                                                                    |
| **Monetized?** | no                                                                                                             |
| **Stack**      | Node.js 20+ CLI with TypeScript and a thin command dispatcher                                                  |

## 2. Problem & validation

- **Problem:** Terminal coding agents often send irrelevant repository context, repeat history, hide token and cost usage, execute oversized loops, and lose project decisions between sessions.

- **Proof it's real:** Pi documents compaction and context thresholds; Aider documents repository mapping; OpenCode documents broad provider support. These validate the category while leaving room for a transparent efficiency layer.

- **Today's workaround:** Developers manually choose files, paste summaries, interrupt agents, inspect provider dashboards, and keep separate notes.

- **Jobs-to-be-done:** When I ask an AI coding agent to change my project, I want it to use only the context needed, work in safe chunks, show what it costs, and remember enough for the next session.

## 3. Users & personas

| Persona                    | Who                                                                               | Top pain                                                               | What success looks like                                               |
| -------------------------- | --------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | --------------------------------------------------------------------- |
| The Cost-Conscious Builder | A solo developer using hosted APIs for daily coding tasks.                        | Repeated context and hidden usage make experiments expensive.          | Completes the same verified tasks with a visible, lower token ledger. |
| The Local-First Developer  | A developer running Ollama or another OpenAI-compatible local endpoint.           | Provider tooling assumes cloud services and inconsistent capabilities. | Uses local models with the same task, memory, and terminal workflow.  |
| The Terminal Team          | A small team that wants repeatable agent sessions without a hosted control plane. | Decisions and task state disappear between sessions.                   | Reloads portable project memory and sees an auditable task history.   |

## 4. MVP scope

Must have (the approval contract):

- [ ] Start an IND session in any supported terminal and detect the project root.
- [ ] Inspect the repository and build a relevance-ranked context set instead of sending the whole tree.
- [ ] Plan a request into bounded task chunks with a visible checkpoint before each execution chunk.
- [ ] Execute file edits and verification commands through a safe tool loop with approval settings.
- [ ] Support a common provider adapter contract for OpenAI-compatible endpoints plus native adapters for OpenAI, Anthropic, and Google.
- [ ] Apply context compression, output truncation, prompt caching where available, and configurable token budgets.
- [ ] Show live input tokens, output tokens, estimated cost, latency, provider, model, savings, and chunk status in the terminal.
- [ ] Persist project memory as Markdown and structured session usage as SQLite, then reload it on the next session.
- [ ] Provide a deterministic benchmark command comparing IND context usage to a full-context baseline.

Should have (if time):

- [ ] Allow model routing by task type, using a cheap model for summaries and a stronger model for difficult edits.
- [ ] Support Ollama discovery and an explicit custom OpenAI-compatible base URL.
- [ ] Offer commands to inspect, edit, export, and reset project memory.
- [ ] Provide JSONL session events for external dashboards without making a dashboard part of the MVP.

Won't do (non-goals):

- Native desktop or mobile applications.
- Hosted accounts, billing, team collaboration, or remote memory sync.
- Unattended destructive commands or autonomous operation without safety controls.
- Every provider-specific feature on the first release.

## 5. User flows

1. User runs ind in a repository, selects or enters a task, reviews the generated plan and context budget, approves a chunk, watches the agent edit files and run verification, then continues or revises the next chunk.
2. User starts a session with a configured local or hosted provider, sees the provider/model and live usage bar, switches model or budget, and completes the task without changing the core workflow.
3. User exits and later runs ind resume; IND loads project memory, recent task state, changed files, and usage history, then proposes the next safe chunk.

## 6. Data model

| Entity         | Key fields                                                                                                                                | Relations                                                                       |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| projects       | id, root_path, display_name, created_at, updated_at                                                                                       | one project has many sessions and memory entries                                |
| sessions       | id, project_id, provider, model, started_at, ended_at, status, input_tokens, output_tokens, estimated_cost                                | many sessions belong to one project                                             |
| chunks         | id, session_id, sequence, title, goal, status, input_tokens, output_tokens, started_at, completed_at                                      | many chunks belong to one session                                               |
| usage_events   | id, session_id, chunk_id, provider, model, event_type, input_tokens, output_tokens, cached_tokens, latency_ms, estimated_cost, created_at | many usage events belong to a session and optional chunk                        |
| memory_entries | id, project_id, category, key, content, source, importance, created_at, updated_at                                                        | many memory entries belong to one project and mirror selected Markdown sections |

## 7. Auth & permissions

- No IND account auth in MVP; provider credentials stay in environment variables or an OS-appropriate local credential store.
- The CLI must never print secret values, write them to project memory, or send them to a model.
- File and command permissions are controlled locally by the user and explicit approval settings.

## 9. Analytics & KPIs

- **KPI:** Median input-token reduction, verified task completion, chunk success rate, session resume success, and usage-ledger completeness.

- **Tools:** Local benchmark JSONL, SQLite usage ledger, and reproducible fixture repositories.

- **Guardrail:** At least 20 percent median input-token reduction with no more than 10 percent regression in verified benchmark completion.


## 10. Risks & open questions

- A common adapter contract may not expose equivalent tool calling, streaming, reasoning, or usage fields across providers.
- Repository indexing can itself consume time and tokens if implemented as a full scan on every turn.
- Estimated cost for local models is not monetary and must be labeled separately from cloud cost.
- Command execution is security-sensitive and needs explicit allow, deny, and approval behavior.

## 11. Decisions (what changed from the raw request)

- Interpret all devices as cross-platform desktop terminals rather than native mobile apps.
- Use a capability-based adapter interface: common chat, streaming, tool call, usage, and cancellation capabilities with graceful fallbacks.
- Use SQLite for metrics and Markdown for portable memory so users can inspect and version the important state.
- Keep the first release free and bring-your-own-key; defer hosted sync and billing.

## 12. AI features

> Rails locked in `stack-blueprint.md` §4.1 from `templates/ai-logic.md`.

| Feature                           | What the user gets                                                                                 | Model                                                                        | Streaming? | Cost rail                                                                                        | Eval cases                                                                       |
| --------------------------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| Repository context selection      | Selects the smallest relevant file and symbol set for a task.                                      | cheap local or fast hosted model with deterministic heuristics first         | no         | Hard input and output caps; prefer indexed metadata and cached summaries over raw file contents. | 10 fixture tasks with expected relevant files and token ceilings.                |
| Task planning and chunk execution | Turns a request into bounded, reviewable coding chunks and produces edits plus verification steps. | cheap model for plan normalization; stronger configured model for hard edits | yes        | Per-chunk budget, maximum tool turns, timeout, cancellation, and model routing policy.           | Golden repository tasks covering add, modify, refactor, test, and bug-fix flows. |
| Memory summarization              | Compresses completed work and decisions into portable project memory.                              | cheap model with a strict schema and deterministic fallback                  | no         | Summarize only changed files and unresolved decisions; never resummarize unchanged memory.       | Continuation tests where a new session must recover decisions and next steps.    |


- **Non-AI fallback:** Use deterministic file relevance rules, Git diff metadata, task chunk templates, and explicit user selection when a model is unavailable or exceeds budget.

- **Kill guardrail for AI:** If context selection does not reduce median input tokens by 20 percent or memory continuation fails in 4 of 5 tests, redesign those subsystems before broadening provider support.


## 13. Design source of truth (from `templates/frontend-design.md`)

- **Source picked:** open-source design pack
- **Tokens:** Dark terminal-first interface with teal accent, semantic green/yellow/red states, compact status bar, explicit token ledger, no decorative panels. Must remain usable without color.
- **Design parity:** screens are visually checked against the source at 375/768/1280 — browser MCP vs Figma/Stitch
- **Validation verdict:** 26/35 → ITERATE — guardrail: Within 30 days of public testing, produce 10 benchmark runs across at least three environments and demonstrate median input-token reduction of at least 20 percent without reducing verified task completion.

---

> **Status: awaiting user approval** — reply **approve** to build, **edit** to revise, or **reject** to stop.