# ticket-flow

Agent ticket system for coding agents. `ticket-flow` owns ticket lifecycle state;
`clawhip` can consume compact `ticket.*` events for routing and delivery.

## Source Of Truth

The source-of-truth boundary is the `ticket-flow` CLI, MCP server, and TypeScript
store API. All ticket creation, mutation, status changes, logs, links, and
checkpoints should go through those surfaces.

The filesystem is the current storage implementation, not the public SSOT
contract:

- store root defaults to `~/.ticket-flow/tickets`
- override the store with `TICKET_FLOW_HOME=/path/to/tickets`
- active tickets live in `active/T-YYYYMMDD-NNN.json`
- closed tickets move to `archive/YYYY-MM/T-YYYYMMDD-NNN.json`
- `index.json` tracks active tickets by ID

Existing external ticket data should be migrated into the `ticket-flow` store
before use. Do not share another product's ticket directory in place as the live
ticket state.

## Usage

```bash
bun install
bun run cli create --title "Fix compaction session recovery" --source discord:123
bun run cli list
bun run cli show T-20260707-001
bun run cli checkpoint T-20260707-001 --phase qa --next-type agent_action --next-command "finish verification"
bun run mcp
```

The MCP server exposes the same ticket mutations as tools, including
`ticket_create`, `ticket_update_status`, `ticket_checkpoint`, and
`ticket_agent_actions`.

## macmini Setup

The current macmini setup uses this repository as the operational ticket-flow
checkout:

```text
/Users/iyen/projects/ticket-flow
branch: codex/remove-openclaw-legacy
```

Wrappers are installed in `~/.local/bin`:

```text
ticket-flow
ticket-flow-mcp
```

They source `~/.config/ticket-flow/env`, which sets:

```bash
TICKET_FLOW_HOME="$HOME/.ticket-flow/tickets"
TICKET_FLOW_REPO_PATH="$HOME/projects/ticket-flow"
TICKET_FLOW_WORKTREE_PATH="$HOME/projects/ticket-flow"
TICKET_FLOW_CLAWHIP=1
TICKET_FLOW_CLAWHIP_URL="http://127.0.0.1:25294"
TICKET_FLOW_CLAWHIP_MODE="best-effort"
```

That means normal macmini use can call:

```bash
ticket-flow create --title "Investigate failing workflow"
ticket-flow list
ticket-flow-mcp
```

## Clawhip Integration

`ticket-flow` projects tickets into compact `ticket.*` events for
[`clawhip`](https://github.com/Yeachan-Heo/clawhip). `clawhip` owns route
matching, templates, mentions, channel choices, sinks, and delivery policy. It
does not create or mutate ticket state.

Manual projection:

```bash
bun run cli clawhip event T-20260707-001 --kind ticket.created --print
bun run cli clawhip event T-20260707-001 --kind ticket.created --send --url http://127.0.0.1:25294
```

Automatic emission is opt-in and best-effort by default:

```bash
TICKET_FLOW_CLAWHIP=1 \
TICKET_FLOW_CLAWHIP_URL=http://127.0.0.1:25294 \
TICKET_FLOW_REPO_PATH="$PWD" \
bun run cli create --title "Notify clawhip"
```

On macmini, `clawhip` is already running as the `com.clawhip.daemon`
LaunchAgent on `http://127.0.0.1:25294`, and the `ticket-flow` wrapper enables
best-effort event emission by default.

See [docs/CLAWHIP-INTEGRATION.md](docs/CLAWHIP-INTEGRATION.md) for the event
contract, environment variables, routing boundaries, failure behavior, and
verification workflow.
