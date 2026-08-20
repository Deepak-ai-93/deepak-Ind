# IND

IND is a cross-platform terminal coding agent focused on lower token usage, bounded task chunks, transparent usage monitoring, and portable project memory.

## Install

### Install directly from GitHub

```bash
npm install -g github:Deepak-ai-93/deepak-Ind
ind --help
```

The GitHub install automatically builds the CLI during installation. IND targets Node.js 20+ on Windows, macOS, and Linux.

### Windows PowerShell troubleshooting

Use the global flag exactly as shown:

```powershell
npm.cmd install --global github:Deepak-ai-93/deepak-Ind
$npmGlobal = npm.cmd prefix --global
& "$npmGlobal\ind.cmd" --help
```

If `ind` is still not recognized, close and reopen PowerShell. The npm global directory must be on PATH. To update the current PowerShell session immediately:

```powershell
$env:Path += ";$(npm.cmd prefix --global)"
ind --help
```

### Install from source

```bash
git clone https://github.com/Deepak-ai-93/deepak-Ind.git
cd deepak-Ind
npm install
npm run build
npm link
ind --help
```

For npm distribution, publish the `ind-terminal` package and install it with `npm install -g ind-terminal`.

## Commands

```text
ind                         Start the interactive surface
ind plan <task>             Preview bounded task chunks
ind context <task>          Preview token-budgeted repository context
ind providers               Show configured provider capabilities
ind discover                Detect Ollama and LM Studio
ind usage                   Show local tokens, cost, latency, and savings
ind memory                  Read project memory
ind memory add <type> <note> Append a memory note
ind memory sync push|pull   Explicitly sync encrypted memory
ind resume                  Show resumable session state
ind benchmark               Run the context savings benchmark
ind leaderboard              Record a reproducible benchmark ranking
ind doctor                  Check environment and likely secrets
```

## Provider configuration

Set provider credentials in the environment or a local secret manager. Never commit `.env` or place credentials in `MEMORY.md`.

```text
IND_PROVIDER=openai-compatible
IND_MODEL=your-model
IND_BASE_URL=http://localhost:11434/v1
OPENAI_API_KEY=...
ANTHROPIC_API_KEY=...
GOOGLE_GENERATIVE_AI_API_KEY=...
```

Supported paths include OpenAI, Anthropic, Google, Ollama, LM Studio, and custom OpenAI-compatible endpoints. Use `ind discover` to probe local runtimes without sending project content.

## Reproducible leaderboard

Run `ind leaderboard` to evaluate the checked-in fixture repositories and write `output/benchmark/leaderboard.md` plus JSONL. Each run includes a fixture-set fingerprint, deterministic run ID, savings, relevance recall, budget compliance, and a weighted score. The command is local-only; sharing the generated JSONL is optional.

## Encrypted memory sync

Memory sync is opt-in. Configure `IND_SYNC_URL` and a secret `IND_SYNC_KEY` (at least 16 characters), then run `ind memory sync push` or `ind memory sync pull`. IND encrypts `MEMORY.md` locally with AES-256-GCM before sending it; HTTPS is required except for localhost testing. The sync service receives only the encrypted envelope, and the key is never written to project files.

## Token efficiency

IND ranks repository files against the task, excludes generated and irrelevant folders, enforces input/output budgets, and records a full-context baseline in `output/benchmark/`. Run:

```bash
ind benchmark
```

The benchmark reports savings and expected-file recall separately; a lower token count does not count as success if relevant files were omitted.

## Local state and safety

- `.ind/usage.db` stores usage events, structured memory, and resume state.
- `MEMORY.md` is human-readable project memory.
- File edits are restricted to the project root and reject `.env` and `.git` paths.
- Commands require approval and block common destructive patterns.
- `ind doctor` scans for likely committed secrets.

## CI test behavior

The test command discovers TypeScript tests without shell globs and runs each file in an isolated Node process. This keeps the native SQLite dependency stable across Windows, macOS, and Linux CI runners.

## Development

```bash
npm install
npm run typecheck
npm test
npm run build
npm pack --dry-run
```

The CI workflow runs typecheck, tests, build, and package verification on Node.js 20.








