# ticket-flow Rust implementation

This folder is an independent Rust implementation of ticket-flow as an
agent-ticket single source of truth. It can run beside the TypeScript package
while the project evaluates a Rust-first path.

## SSOT Rules

- All ticket creation, mutation, status changes, checkpoints, imports, links,
  logs, handoffs, approvals, evidence, views, and context packs go through the
  Rust CLI, MCP server, or `ticket-flow-core` store API.
- Existing ticket stores are imported into ticket-flow. They are not shared in
  place.
- clawhip consumes ticket-flow events. It does not create or own ticket state.
- The filesystem is the store implementation: `active/`, `archive/YYYY-MM/`,
  `events/`, and `index.json`.

## CLI

```sh
cargo run -p ticket-flow-cli -- create --title "Investigate issue"
cargo run -p ticket-flow-cli -- list --format json
cargo run -p ticket-flow-cli -- checkpoint T-20260708-001 --phase qa --next-type agent_action
cargo run -p ticket-flow-cli -- import /path/to/old-ticket-store
```

Set `TICKET_FLOW_HOME` to choose the store root. Without it the CLI uses
`$HOME/.ticket-flow/tickets`.

## MCP

The Rust stdio MCP server is exposed as `ticket-flow-mcp`.

```sh
cargo run -p ticket-flow-cli --bin ticket-flow-mcp
```

Implemented tools:

- `ticket_create`
- `ticket_list`
- `ticket_get`
- `ticket_import`
- `ticket_update_status`
- `ticket_link`
- `ticket_add_log`
- `ticket_checkpoint`
- `ticket_agent_actions`

## Import And Archive Layout

`ticket-flow import <source-root>` reads `active/*.json` and
`archive/YYYY-MM/*.json`, rejects duplicate source IDs, rejects destination
collisions before writing, copies non-`done` tickets into `active/`, copies
`done` tickets into `archive/YYYY-MM/`, updates `index.json`, and appends a
`ticket.imported` event for each ticket.

Archival status transitions also write to `archive/YYYY-MM/`. The reader keeps
compatibility with older flat `archive/<ticket-id>.json` files.

## clawhip

Explicit event replay:

```sh
cargo run -p ticket-flow-cli -- clawhip event T-20260708-001 --kind ticket.created --print
cargo run -p ticket-flow-cli -- clawhip event T-20260708-001 --kind ticket.created --send --url http://127.0.0.1:25294
```

Opt-in auto delivery:

```sh
TICKET_FLOW_CLAWHIP=1 \
TICKET_FLOW_CLAWHIP_URL=http://127.0.0.1:25294 \
TICKET_FLOW_REPO_PATH=/path/to/repo \
cargo run -p ticket-flow-cli -- create --title "Route to clawhip"
```

Auto delivery is best-effort: ticket mutations still succeed when clawhip is
down. Explicit `--send` is strict and exits non-zero on delivery failure.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
