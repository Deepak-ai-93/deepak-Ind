# IND (Rust Native)

> **A high-performance, zero-dependency, ultra-low memory terminal coding agent built in Rust.**

IND is designed for standalone execution across Windows, macOS, and Linux. It delivers token-efficient context indexing, bounded task execution, local SQLite usage ledger, security auditing, and reproducible benchmarks.

---

## ⚡ Features

- **🚀 Native Rust Performance**: Single binary, zero external runtime dependencies (no Node.js/Python required), < 10MB memory footprint.
- **📉 Intelligent Token Reduction**: Token-budgeted context selection achieving **~50% input token savings** without degrading task completion quality.
- **🔒 Built-in Security Scanner (`ind doctor`)**: Proactively scans for leaked API keys, unignored `.env` files, sensitive credential leaks, and permission violations.
- **🤖 Neutral Provider Support**: First-class streaming support for OpenAI, Anthropic, Google Gemini, Ollama, LM Studio, and any custom OpenAI-compatible endpoint.
- **🎛️ Tiered Model Routing**: Automatically classifies incoming tasks and routes between cheap and strong model tiers for optimal cost-efficiency.
- **🛡️ Team Policies & Safe Execution**: Restrict allowed commands and providers via `.ind/policy.json` with auto-approval or interactive approval modes.
- **📊 SQLite Usage & Cost Ledger**: Local persistent tracking of prompt tokens, completion tokens, latency, and estimated cost savings.
- **🧠 Portable Project Memory**: Append-only `MEMORY.md` and encrypted sync capabilities.
- **🏆 Reproducible Benchmarks**: Built-in benchmark harness and leaderboard generation (`ind benchmark`, `ind leaderboard`).

---

## 📦 Installation

### Option 1: Download Prebuilt Binaries (Recommended)

Download the latest standalone executable from [GitHub Releases](https://github.com/Deepak-ai-93/deepak-Ind/releases):

| OS / Architecture | Artifact Archive | Binary Name |
|---|---|---|
| **Windows** (x86_64) | `ind-windows-x86_64.zip` | `ind.exe` |
| **Linux** (x86_64) | `ind-linux-x86_64.tar.gz` | `ind` |
| **Linux** (ARM64 / aarch64) | `ind-linux-aarch64.tar.gz` | `ind` |
| **macOS** (Apple Silicon M1/M2/M3/M4) | `ind-macos-aarch64.tar.gz` | `ind` |
| **macOS** (Intel x86_64) | `ind-macos-x86_64.tar.gz` | `ind` |

Extract the archive and move `ind` (or `ind.exe`) into your system's `PATH`.

---

### Option 2: Install via Cargo (from Source)

If you have the Rust toolchain installed:

```bash
# Clone the repository
git clone https://github.com/Deepak-ai-93/deepak-Ind.git
cd deepak-Ind

# Install directly to ~/.cargo/bin (must be in your PATH)
cargo install --path .
```

---

### Option 3: Build from Source

```bash
# Clone repository
git clone https://github.com/Deepak-ai-93/deepak-Ind.git
cd deepak-Ind

# Build release binary
cargo build --release

# The compiled binary is located at:
# target/release/ind       (Linux/macOS)
# target/release/ind.exe   (Windows)
```

---

## ⚙️ Configuration & Provider Setup

IND reads configuration from environment variables or `.ind/config.json`.

### Environment Variables

Set your provider credentials in your terminal session or a local environment file:

```bash
# --- Provider API Keys ---
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_GENERATIVE_AI_API_KEY="..."

# --- Provider Configuration ---
# Options: openai-compatible (default), openai, anthropic, google
export IND_PROVIDER="openai-compatible"

# Endpoint (default for local Ollama: http://127.0.0.1:11434/v1)
export IND_BASE_URL="http://127.0.0.1:11434/v1"

# Models
export IND_MODEL="llama3"
export IND_CHEAP_MODEL="llama3"
export IND_STRONG_MODEL="llama3"

# Approval mode: chunk (default), command, or never
export IND_APPROVAL="chunk"
```

*On Windows PowerShell, use `$env:OPENAI_API_KEY="sk-..."`.*

---

### Project Configuration (`.ind/config.json`)

Create an optional `.ind/config.json` in your project root for project-level overrides:

```json
{
  "provider": "openai-compatible",
  "baseUrl": "http://127.0.0.1:11434/v1",
  "model": "qwen2.5-coder",
  "cheapModel": "qwen2.5-coder:7b",
  "strongModel": "qwen2.5-coder:32b",
  "routing": "auto",
  "approval": "chunk",
  "maxInputTokens": 12000,
  "maxOutputTokens": 4000,
  "maxToolTurns": 8
}
```

---

### Team Policy Rules (`.ind/policy.json`)

Enforce organizational safety constraints on allowed providers and commands:

```json
{
  "approval": "chunk",
  "allowedProviders": ["openai-compatible", "anthropic"],
  "allowedCommands": ["^cargo (test|build|check)$", "^npm (test|run)"],
  "deniedCommands": ["rm -rf", "curl.*\\|.*sh", "format"]
}
```

---

## 🛠️ CLI Usage & Command Reference

```text
ind [COMMAND] [TASK...]
```

### Core Commands

| Command | Description | Example |
|---|---|---|
| `ind "<task>"` | Execute an interactive task directly | `ind "fix the auth token validation"` |
| `ind run <task>` | Execute task with bounded, approved chunks | `ind run "add integration tests for payment"` |
| `ind plan <task>` | Preview 3-chunk execution plan without running | `ind plan "refactor logger to use tracing"` |
| `ind context <task>` | Preview token-budgeted file selection | `ind context "update login form styling"` |
| `ind route <task>` | Inspect task routing (cheap vs. strong model) | `ind route "explain how memory sync works"` |
| `ind budget <task>` | Estimate tokens, turns, savings, and cost | `ind budget "migrate database schema"` |

---

### Diagnostics, Providers & Security

| Command | Description |
|---|---|
| `ind doctor` | Run comprehensive diagnostics (toolchain, keys, secret leak audit, config validity) |
| `ind providers` | View currently configured provider capabilities and model parameters |
| `ind discover` | Automatically detect local running LLMs (Ollama, LM Studio) |
| `ind config` | Display the resolved project configuration |
| `ind policy` | View active team policies and allowed/denied command patterns |

---

### Memory, Usage & Telemetry

| Command | Description |
|---|---|
| `ind usage` | Display SQLite token usage statistics, lifetime cost, and savings |
| `ind memory` | View project context and conventions from `MEMORY.md` |
| `ind memory add <category> <note>` | Append a persistent decision or pattern to project memory |
| `ind resume` | Check and resume previously interrupted session state |

---

### Benchmarking

| Command | Description |
|---|---|
| `ind benchmark` | Run context savings benchmarks across test fixtures |
| `ind leaderboard` | Generate reproducible markdown & JSONL reports in `output/benchmark/` |

---

## 🧪 Development & Testing

```bash
# Run unit & integration test suite (46 tests)
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
