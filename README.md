# ticket-flow

Standalone OpenClaw ticket CLI and MCP server.

This repository intentionally preserves the existing OpenClaw ticket JSON contract:

- store root defaults to `~/.openclaw/tickets`
- active tickets live in `active/T-YYYYMMDD-NNN.json`
- closed tickets move to `archive/YYYY-MM/T-YYYYMMDD-NNN.json`
- `index.json` tracks active tickets by ID

The service boundary is the TypeScript domain and store layer. The CLI and MCP server are adapters on top of the same behavior.

## Usage

```bash
bun install
bun run cli create --title "Fix compaction session recovery" --source discord:123
bun run cli list
bun run mcp
```

Override the store path with `OPENCLAW_TICKET_HOME=/path/to/tickets`.
