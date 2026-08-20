# IND & Pi (π) — Native Rust AI Coding Terminal Agent

> **A high-performance, zero-dependency, ultra-low memory autonomous AI coding agent & interactive REPL built in Rust.**

**IND** (and its companion binary **`pi`**) provides an intelligent, agentic terminal coding assistant that can read/edit your repository, execute tests and commands, and auto-fix code with surgical accuracy while cutting token usage by **~50%**.

---

## ⚡ Key Highlights

- **🤖 Interactive AI Coding REPL (`pi` or `ind chat`)**: Full interactive chat interface with streaming markdown responses, syntax highlighting, and slash commands.
- **🚀 Native Rust Performance**: Single standalone binary, zero external runtime dependencies (no Node.js/Python required), < 10MB memory footprint.
- **🛠️ Autonomous Agent Tool Loop**: Models can iteratively inspect files (`read_file`), write code (`write_file`), browse directories (`list_files`), and run builds/tests (`run_command`) with auto-fix capabilities.
- **📉 Intelligent Token Reduction**: Token-budgeted context selection achieving **~50% input token savings** without degrading task completion quality.
- **🔒 Built-in Security Scanner (`ind doctor`)**: Proactively scans for leaked API keys, unignored `.env` files, sensitive credential leaks, and permission violations.
- **🌐 Neutral Provider Support**: First-class streaming support for OpenAI, Anthropic Claude, Google Gemini, Ollama, LM Studio, and any custom OpenAI-compatible endpoint.
- **🎛️ Tiered Model Routing**: Automatically classifies incoming tasks and routes between cheap and strong model tiers for optimal cost-efficiency.
- **🛡️ Team Policies & Safe Execution**: Restrict allowed commands and providers via `.ind/policy.json` with auto-approval or interactive approval modes.
- **📊 SQLite Usage & Cost Ledger**: Local persistent tracking of prompt tokens, completion tokens, latency, and estimated cost savings.
- **🧠 Portable Project Memory**: Append-only `MEMORY.md` and encrypted sync capabilities.
- **🏆 Reproducible Benchmarks**: Built-in benchmark harness and leaderboard generation (`ind benchmark`, `ind leaderboard`).

---

## 📦 Installation & Setup

Installing builds both the **`ind`** CLI and the **`pi`** interactive coding assistant.

### Option 1: Install via Cargo (One-Liner)

```bash
# Clone and install directly to ~/.cargo/bin (must be in your PATH)
git clone https://github.com/Deepak-ai-93/deepak-Ind.git
cd deepak-Ind
cargo install --path .
```

After installation, both `ind` and `pi` commands will be immediately available in your terminal!

---

### Option 2: Download Prebuilt Binaries

Download the standalone release archive from [GitHub Releases](https://github.com/Deepak-ai-93/deepak-Ind/releases):

| OS / Architecture | Artifact Archive | Included Binaries |
|---|---|---|
| **Windows** (x86_64) | `ind-windows-x86_64.zip` | `ind.exe`, `pi.exe` |
| **Linux** (x86_64) | `ind-linux-x86_64.tar.gz` | `ind`, `pi` |
| **Linux** (ARM64 / aarch64) | `ind-linux-aarch64.tar.gz` | `ind`, `pi` |
| **macOS** (Apple Silicon M1/M2/M3/M4) | `ind-macos-aarch64.tar.gz` | `ind`, `pi` |
| **macOS** (Intel x86_64) | `ind-macos-x86_64.tar.gz` | `ind`, `pi` |

Extract the archive and add the binaries to your system's `PATH`.

---

### Option 3: Build from Source

```bash
git clone https://github.com/Deepak-ai-93/deepak-Ind.git
cd deepak-Ind

# Build optimized release binaries
cargo build --release

# The compiled binaries will be in:
# target/release/ind and target/release/pi       (Linux/macOS)
# target/release/ind.exe and target/release/pi.exe (Windows)
```

---

## ⚙️ Provider Configuration

Configure your credentials using environment variables or a local `.ind/config.json`:

### Environment Variables

```bash
# Provider API Keys
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_GENERATIVE_AI_API_KEY="..."

# Provider selection (openai-compatible, openai, anthropic, google)
export IND_PROVIDER="openai-compatible"

# Endpoint (default for Ollama: http://127.0.0.1:11434/v1)
export IND_BASE_URL="http://127.0.0.1:11434/v1"

# Models
export IND_MODEL="qwen2.5-coder"
export IND_CHEAP_MODEL="qwen2.5-coder:7b"
export IND_STRONG_MODEL="qwen2.5-coder:32b"
```

*On Windows PowerShell, use `$env:OPENAI_API_KEY="sk-..."`.*

---

## 🚀 Interactive Pi REPL (`pi` or `ind chat`)

Start the interactive coding assistant by simply typing `pi` or `ind`:

```bash
# Launch interactive AI coding session
pi

# Or via ind
ind chat
ind repl
ind
```

### Slash Commands in REPL

Inside the `pi > ` prompt, use these slash commands for quick actions:

| Slash Command | Description |
|---|---|
| `/help`, `/h` | Display available slash commands and cheat sheet |
| `/clear` | Clear conversation history & start a fresh session |
| `/compact` | Prune older turns to preserve context token budget |
| `/model <name>` | Switch active model on the fly (e.g. `/model claude-3-7-sonnet`) |
| `/plan <task>` | Generate and preview a bounded 3-chunk execution plan |
| `/doctor` | Run project diagnostics, secret leak audit, and toolchain checks |
| `/usage` | View SQLite token usage ledger, session cost, and savings |
| `/memory` | Inspect active project conventions from `MEMORY.md` |
| `/diff` | Show uncommitted Git diff in the repository |
| `/exit`, `/quit` | Exit the interactive session |

---

## 🛠️ CLI Command Reference (`ind`)

```text
ind [COMMAND] [TASK...]
```

### 1. Agentic Coding & Planning

| Command | Description | Example |
|---|---|---|
| `pi` | Launch the interactive AI coding terminal REPL | `pi` |
| `ind chat` / `ind repl` | Launch the interactive AI coding terminal REPL | `ind chat` |
| `ind "<task>"` | Execute an AI task directly | `ind "fix the auth token validation"` |
| `ind run <task>` | Execute task with bounded, approved chunks | `ind run "add integration tests for payment"` |
| `ind plan <task>` | Preview 3-chunk execution plan without running | `ind plan "refactor logger to use tracing"` |
| `ind context <task>` | Preview token-budgeted file selection | `ind context "update login form styling"` |
| `ind route <task>` | Inspect task routing (cheap vs. strong model) | `ind route "explain how memory sync works"` |
| `ind budget <task>` | Estimate tokens, turns, savings, and cost | `ind budget "migrate database schema"` |

---

### 2. Diagnostics, Security & Configuration

| Command | Description |
|---|---|
| `ind doctor` | Run comprehensive diagnostics (toolchain, keys, secret leak audit, config validity) |
| `ind providers` | View currently configured provider capabilities and model parameters |
| `ind discover` | Automatically detect local running LLMs (Ollama, LM Studio) |
| `ind config` | Display the resolved project configuration |
| `ind policy` | View active team policies and allowed/denied command patterns |

---

### 3. Memory, Usage & Benchmarking

| Command | Description |
|---|---|
| `ind usage` | Display SQLite token usage statistics, lifetime cost, and savings |
| `ind memory` | View project context and conventions from `MEMORY.md` |
| `ind memory add <category> <note>` | Append a persistent decision or pattern to project memory |
| `ind resume` | Check and resume previously interrupted session state |
| `ind benchmark` | Run context savings benchmarks across test fixtures |
| `ind leaderboard` | Generate reproducible markdown & JSONL reports in `output/benchmark/` |

---

## 🧪 Development & Testing

```bash
# Run unit & integration test suite (48 tests)
cargo test

# Check code formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets -- -D warnings

# Build optimized binary
cargo build --release
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
