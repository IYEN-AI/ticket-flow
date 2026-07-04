# ticket-flow

Agent-oriented local ticket system with CLI, MCP, and event-routing adapters.

The storage layer intentionally preserves the existing OpenClaw-compatible
ticket JSON contract for migration and interoperability:

- store root defaults to `~/.openclaw/tickets`
- active tickets live in `active/T-YYYYMMDD-NNN.json`
- closed tickets move to `archive/YYYY-MM/T-YYYYMMDD-NNN.json`
- `index.json` tracks active tickets by ID

The service boundary is the TypeScript domain and store layer. The CLI, MCP
server, and `clawhip` event projection are adapters on top of the same behavior.

## Usage

```bash
bun install
bun run cli create --title "Fix compaction session recovery" --source discord:123
bun run cli list
bun run mcp
```

Override the store path with `TICKET_FLOW_HOME=/path/to/tickets`.
`OPENCLAW_TICKET_HOME` is still supported as a backward-compatible alias.

## Clawhip integration

`ticket-flow` can project tickets into compact `ticket.*` events for `clawhip` without
changing the ticket JSON contract.

```bash
bun run cli clawhip event T-20260704-001 --kind ticket.created --print
bun run cli clawhip event T-20260704-001 --kind ticket.created --send --url http://127.0.0.1:25294
```

Automatic emission is opt-in and best-effort by default:

```bash
TICKET_FLOW_CLAWHIP=1 \
TICKET_FLOW_CLAWHIP_URL=http://127.0.0.1:25294 \
TICKET_FLOW_REPO_PATH="$PWD" \
bun run cli create --title "Notify clawhip"
```

`clawhip` owns routing, templates, mentions, and sinks. A local verification route:

```toml
[[routes]]
event = "ticket.*"
filter = { provider = "ticket-flow", repo_path = "*/ticket-flow" }
sink = "localfile"
local_path = "/tmp/ticket-flow-clawhip-events.jsonl"
format = "raw"
template = "{ticket_id} [{status}] {title}"
allow_dynamic_tokens = true
```

See [docs/CLAWHIP-INTEGRATION.md](docs/CLAWHIP-INTEGRATION.md) for the full
event contract, environment variables, routing boundaries, failure behavior, and
verification workflow.
