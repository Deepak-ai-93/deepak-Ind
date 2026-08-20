# Validation â€” IND

## Scorecard

| Criterion | Score / 5 | Reason |
|---|---:|---|
| Problem clarity | 5 | Token waste, opaque execution, and lost context are concrete workflow pains. |
| Market reachability | 4 | GitHub, npm, and AI/CLI communities are reachable without paid acquisition. |
| Competition | 3 | Pi, Aider, and OpenCode already cover large parts of the workflow. |
| Monetization | 3 | No v1 billing, but open-core plus future hosted sync/team controls is plausible. |
| Feasibility | 4 | A focused CLI can start with an OpenAI-compatible adapter and local SQLite. |
| Moat | 4 | Measured context efficiency, benchmark data, and portable memory can compound, but are not defensible yet. |
| Time to MVP | 3 | Cross-platform support plus multiple providers increases integration and QA work. |

**Total: 24/35 â€” ITERATE**

## Kill criteria

- If IND cannot demonstrate at least 20% input-token reduction on the benchmark while preserving task completion, revisit the core strategy.
- If provider abstraction causes unreliable tool calls across the first three adapters, narrow the common capability contract before adding more providers.
- If memory reloads are not useful in at least 4 of 5 continuation tests, treat memory as a redesign item rather than shipping it as a checkbox.

## Unit economics sanity

The MVP has no hosted inference cost if users bring their own keys or run locally. Hosted usage costs belong to the user/provider account. Future hosted features must meter storage and sync separately from model inference.

## Guardrail

Within the first 30 days of public testing, IND should have 10 benchmark runs from at least three environments and a median input-token reduction of 20% or more. Otherwise, iterate on context selection and compaction before expanding the provider matrix.

## Verdict

**ITERATE before broad implementation.** The product is feasible and differentiated enough to build an MVP, but the key claim must be proven with a repeatable benchmark and not treated as marketing language. Monetization is deliberately deferred from v1.
