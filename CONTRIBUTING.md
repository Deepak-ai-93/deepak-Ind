# Contributing to IND

Thanks for helping improve IND.

## Development setup

Requirements: Node.js 20 or newer.

```bash
npm install
npm run typecheck
npm test
npm run build
```

Keep changes focused, add tests for behavior changes, and preserve cross-platform support on Windows, macOS, and Linux.

## Pull requests

- Explain the user-facing behavior and safety impact.
- Include verification commands and their results.
- Do not commit API keys, `.env` files, `.ind/` state, generated `dist/`, or benchmark output.
- Changes to command execution, provider access, memory, or encryption need tests and documentation.

## Provider integrations

Provider adapters must normalize streaming events, usage, cancellation, and errors behind the shared contract. Never log credentials or project content by default.
