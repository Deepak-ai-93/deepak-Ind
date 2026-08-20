# TODO — IND

> **Confirmed:** YES · by: user · on: 2026-08-20
> The build may **NOT** start until the user approves this list (SKILL.md Stage 3 gate).
> Manage: `node scripts/todo.mjs list | add "task" --p P1 | priority <id> P0 | done <id> | blocked <id> | confirm`
> Scope: IND MVP: a provider-neutral, token-efficient, chunked terminal coding agent with live usage telemetry and project memory.

## P0 — do first
_none yet_

## P1 — important
_none yet_

## P2 — nice to have
_none yet_

## Done
- [x] (P2) #15 A community benchmark leaderboard based on reproducible fixture repositories. (from the interview)
- [x] (P2) #14 Optional hosted memory sync with end-to-end encryption. (from the interview)
- [x] (P2) #13 Team policy files for command approvals and provider restrictions. (from the interview)
- [x] (P2) #12 A replay mode for comparing provider and model choices on the same task. (from the interview)
- [x] (P2) #11 A token budget planner that predicts cost before a chunk runs. (from the interview)
- [x] (P2) #10 Add advanced routing and optional JSONL integrations — ref: blueprint §6.10
- [x] (P1) #9 Harden cross-platform packaging, security, tests, and documentation — ref: blueprint §6.9
- [x] (P1) #8 Create benchmark fixtures and baseline comparison — ref: blueprint §6.8
- [x] (P1) #7 Add native provider adapters and local runtime discovery — ref: blueprint §6.7
- [x] (P0) #6 Add Markdown plus SQLite project memory and resume — ref: blueprint §6.6
- [x] (P0) #5 Add SQLite usage ledger and live terminal monitor — ref: blueprint §6.5
- [x] (P0) #4 Implement chunked task planning, approval, tools, edits, and verification — ref: blueprint §6.4
- [x] (P0) #3 Build repository inspection and token-efficient context selection — ref: blueprint §6.3
- [x] (P0) #2 Implement provider adapter contract and OpenAI-compatible endpoint — ref: blueprint §6.2
- [x] (P0) #1 Scaffold the cross-platform TypeScript CLI and configuration system — ref: blueprint §6.1

---

```
- [ ] (P1) #N Example task — ref: PRD-4
```

- **Status:** `[ ]` todo · `[~]` doing · `[!]` blocked · `[x]` done

- **Priority:** `(P0)` do first · `(P1)` important · `(P2)` nice to have

- **ID:** `#n` — assigned by `todo.mjs add`, never reused; used by `priority/done/doing/blocked/todo/remove`

- **Reference:** `— ref: PRD-4` links a task to the PRD/blueprint (optional)

- **The user owns the list.** The agent proposes it; the user **confirms** it (gate), adds tasks, and changes priorities at any time — even mid-build.

- **Priorities:** `P0` = do first (blocks everything else) · `P1` = important · `P2` = nice to have. Order within a group is the suggested build order.

- **Confirm:** the agent runs `node scripts/todo.mjs confirm` only after the user explicitly approves. The build pack (`PRD.md` + `stack-blueprint.md` + `sitemap.md`) and this list are confirmed together — no code before both are approved.

- **Done only when verified:** a task is `done` after it runs and is verified — not when the code is merely written.

- **Script owns the four sections** (`P0/P1/P2/Done`): `todo.mjs` re-sorts tasks into them on every change. Keep prose notes **above the list** or **under the Done section**.
