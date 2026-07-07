# Verification

## Automated Gates

```bash
bun run typecheck
bun test
bun run lint
```

Latest result:

- TypeScript: passed
- Tests: 19 passed, 0 failed
- Biome: passed over 43 files

## Manual CLI QA

Store:

```bash
/tmp/ticket-flow-qa
```

Commands exercised:

```bash
TICKET_FLOW_HOME=/tmp/ticket-flow-qa bun run src/cli.ts create --title 'QA ticket' --type feature --priority high --goal 'prove CLI surface' --acceptance 'ticket json exists' --tag qa,mcp --source discord:999
TICKET_FLOW_HOME=/tmp/ticket-flow-qa bun run src/cli.ts status T-20260701-001 doing --note 'start QA'
TICKET_FLOW_HOME=/tmp/ticket-flow-qa bun run src/cli.ts link T-20260701-001 --thread 123456
TICKET_FLOW_HOME=/tmp/ticket-flow-qa bun run src/cli.ts log T-20260701-001 'manual QA note'
TICKET_FLOW_HOME=/tmp/ticket-flow-qa bun run src/cli.ts checkpoint T-20260701-001 --phase qa --next-type agent_action --next-command 'finish verification' --next-owner iyen --note 'handoff packet'
TICKET_FLOW_HOME=/tmp/ticket-flow-qa bun run src/cli.ts agent-actions
```

Observed output included:

```text
T-20260701-001: open -> doing
T-20260701-001: linked threads -> 123456
T-20260701-001: logged
T-20260701-001: checkpoint logged
T-20260701-001 [doing] owner=iyen age=0m :: finish verification
```

## Manual MCP QA

Command shape:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"manual-qa","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ticket_create","arguments":{"title":"MCP QA ticket","priority":"low","type":"chore","source":"qa:mcp"}}}' \
  | TICKET_FLOW_HOME=/tmp/ticket-flow-mcp-qa bun run src/mcp.ts
```

Observed:

- `initialize` returned protocol version `2025-06-18`.
- `tools/list` returned all eight ticket tools.
- `ticket_create` returned `T-20260701-001` with existing JSON contract fields.
- `ticket_agent_actions` returned an empty JSON array on an empty active-action store.
- stdout contained JSON-RPC messages only.

## Compatibility QA

The existing shell CLI accepts both `--source type:ref` and `--source type ref`. The new CLI was checked with the split form:

```bash
TICKET_FLOW_HOME=/tmp/ticket-flow-source-qa bun run src/cli.ts create --title 'source split' --source discord 777
```

Observed JSON:

```json
{
  "source": {
    "type": "discord",
    "ref": "777"
  }
}
```

Historical data sweep:

```bash
bun --eval '...TicketSchema.safeParse over /Users/iyen/.ticket-flow/tickets/{active,archive}/**/*.json...'
```

Observed:

```json
{"total":346,"failed":0}
```

Default-store read-only list:

```bash
TICKET_FLOW_HOME=/Users/iyen/.ticket-flow/tickets bun run src/cli.ts list | head -20
```

Observed: command exited 0 and printed active tickets from the real store.

Empty checkpoint compatibility:

```bash
TICKET_FLOW_HOME=/tmp/ticket-flow-empty-checkpoint bun run src/cli.ts checkpoint T-20260701-001
```

Observed: command exited 1 with `checkpoint requires at least one field`, matching the existing shell guardrail.

Store override compatibility:

- `TICKET_FLOW_HOME` is the only store override.
- Old product-specific store override env vars are ignored.
- With no override, the default store is `~/.ticket-flow/tickets`.

## Manual Clawhip QA

The `clawhip` integration was verified against a real `clawhip v0.6.11` daemon
using a `ticket.*` localfile route.

Commands exercised:

```bash
TICKET_FLOW_REPO_PATH="$PWD" bun run cli clawhip event T-20260704-001 --kind ticket.created --print
TICKET_FLOW_REPO_PATH="$PWD" bun run cli clawhip event T-20260704-001 --kind ticket.created --send --url http://127.0.0.1:25343
TICKET_FLOW_CLAWHIP=1 TICKET_FLOW_CLAWHIP_URL=http://127.0.0.1:25343 TICKET_FLOW_REPO_PATH="$PWD" bun run cli create --title "Auto emit to clawhip"
```

Observed:

- manual projection printed routeable `ticket.created` IncomingEvent JSON
- manual send returned `clawhip: sent ticket.created status=202`
- automatic create appended `ticket.created` JSON to the configured localfile sink
- payloads contained route fields such as `provider`, `ticket_id`, `repo_path`,
  and `repo_name`
- payloads did not include raw ticket logs, artifacts, goals, or acceptance text

Evidence:

```text
.omo/ulw-implementation/20260704-clawhip-integration/evidence/real-surface/REAL-SURFACE-QA.md
.omo/ulw-implementation/20260704-clawhip-integration/evidence/real-surface/cli-print.json
.omo/ulw-implementation/20260704-clawhip-integration/evidence/real-surface/localfile.jsonl
```
