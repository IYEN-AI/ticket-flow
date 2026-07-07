# Clawhip Integration

`ticket-flow` can emit compact `ticket.*` events to a running
[`clawhip`](https://github.com/Yeachan-Heo/clawhip) daemon. The integration keeps
the local ticket JSON store as the source of truth and uses `clawhip` only as
the event routing layer.

## Responsibilities

`ticket-flow` owns:

- ticket creation, status changes, checkpoints, and the active/archive JSON store
- projection from a ticket record into a compact event payload
- best-effort delivery to the configured `clawhip` daemon

`clawhip` owns:

- route matching
- sink selection
- message templates
- mentions, channel choices, and delivery policies

The integration intentionally does not persist `clawhip` delivery state in ticket
JSON, and it does not send raw logs, artifacts, goals, or acceptance criteria.

## Event Sources

Automatic emission is opt-in. When enabled, these store mutations emit events
after the ticket file and index have been updated:

| Ticket-flow action | Emitted event |
| --- | --- |
| `createTicket` | `ticket.created` |
| `updateStatus` | `ticket.status_changed` |
| `checkpointTicket` | `ticket.checkpointed` |
| `checkpointTicket` with `next_action.type = "agent_action"` | `ticket.agent_action_available` |
| `checkpointTicket` with a blocker or blocked next action | `ticket.blocked` |

`ticket.review_ready` is part of the supported event contract for manual
projection, but no automatic store mutation currently emits it.

## Event Shape

Events are sent as `clawhip` IncomingEvent JSON:

```json
{
  "type": "ticket.created",
  "payload": {
    "provider": "ticket-flow",
    "event": "ticket.created",
    "ticket_id": "T-20260704-001",
    "title": "Notify clawhip",
    "status": "open",
    "summary": "T-20260704-001 [open] Notify clawhip",
    "repo_path": "/repo/ticket-flow",
    "worktree_path": "/repo/ticket-flow",
    "repo_name": "ticket-flow",
    "event_timestamp": "2026-07-04T00:00:00.000Z"
  }
}
```

Common payload fields:

| Field | Description |
| --- | --- |
| `provider` | Always `ticket-flow`; useful for route filters. |
| `event` | Same value as the top-level `type`. |
| `ticket_id` | `ticket-flow` ticket ID in the `T-YYYYMMDD-NNN` format. |
| `title` | Ticket title. |
| `status` | Current ticket status. |
| `priority`, `type`, `assignee` | Present when the ticket has those values. |
| `summary` | Compact display string: `<id> [<status>] <title>`. |
| `event_timestamp` | Ticket `updated` timestamp after the mutation. |
| `repo_path`, `worktree_path`, `repo_name` | Present when configured through env or CLI projection. |

Contextual fields:

| Field | When present |
| --- | --- |
| `from_status`, `to_status` | `ticket.status_changed`. |
| `phase`, `decision`, `blocker`, `next` | Copied from `ticket.current` when present. |
| `next_action_type`, `next_action_command`, `next_action_owner` | Copied from `ticket.current.next_action`. |
| `question_summary` | `ticket.blocked` and `ticket.review_ready`, derived from blocker, next, or title. |
| `correlation_id` | Present when supplied to the projection API. |

Excluded fields:

- full ticket JSON
- `goal`
- `acceptance`
- `log`
- `artifacts`
- `links`
- raw source payloads beyond routeable summary fields

## Manual Projection

Use the CLI helper to inspect the exact event JSON before enabling automatic
delivery:

```bash
TICKET_FLOW_REPO_PATH="$PWD" \
bun run cli clawhip event T-20260704-001 --kind ticket.created --print
```

Send one event to a daemon:

```bash
TICKET_FLOW_REPO_PATH="$PWD" \
bun run cli clawhip event T-20260704-001 \
  --kind ticket.created \
  --send \
  --url http://127.0.0.1:25294
```

If `--send` is omitted, the command prints JSON. If `--send` is provided with
`--print`, it prints and sends the same event.

Unsupported event kinds are rejected by the same schema used by the projection
module.

## Automatic Delivery

Automatic emission is disabled unless `TICKET_FLOW_CLAWHIP` is enabled:

```bash
TICKET_FLOW_CLAWHIP=1 \
TICKET_FLOW_CLAWHIP_URL=http://127.0.0.1:25294 \
TICKET_FLOW_REPO_PATH="$PWD" \
bun run cli create --title "Notify clawhip"
```

Environment variables:

| Variable | Default | Description |
| --- | --- | --- |
| `TICKET_FLOW_CLAWHIP` | unset | Enables automatic emission when set to `1`, `true`, `yes`, or `on`. |
| `TICKET_FLOW_CLAWHIP_URL` | `http://127.0.0.1:25294` | Base daemon URL. `/event` is appended when the path does not already end with `/event`. |
| `TICKET_FLOW_CLAWHIP_MODE` | `best-effort` | Set to `strict` to rethrow delivery failures. Any other value is best-effort. |
| `TICKET_FLOW_CLAWHIP_TIMEOUT_MS` | `1000` | Positive integer timeout for sending. Invalid values fall back to `1000`. |
| `TICKET_FLOW_REPO_PATH` | unset | Adds `repo_path` and `repo_name` to the payload. |
| `TICKET_FLOW_WORKTREE_PATH` | `TICKET_FLOW_REPO_PATH` | Adds `worktree_path` to the payload. |

In default best-effort mode, a down or unreachable `clawhip` daemon does not fail
the ticket mutation. Use strict mode only for operator workflows where delivery
failure should abort the command.

## Clawhip Route Example

Minimal localfile route:

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

Because the payload keeps routing fields flat, `clawhip` route filters and
templates do not need to understand the full ticket schema.

## Verification

Local automated checks:

```bash
bun test
bun run typecheck
bun run lint
```

Real-surface smoke check with a running `clawhip` daemon:

1. Start `clawhip` with a `ticket.*` localfile route.
2. Run `bun run cli clawhip event <ticket-id> --kind ticket.created --print`.
3. Run the same command with `--send --url <daemon-url>`.
4. Enable `TICKET_FLOW_CLAWHIP=1` and create a ticket.
5. Confirm the localfile sink contains both manual and automatic
   `ticket.created` events.

The implementation evidence for this change is recorded under
`.omo/ulw-implementation/20260704-clawhip-integration/evidence/`, including a
real daemon smoke test in `real-surface/REAL-SURFACE-QA.md`.
