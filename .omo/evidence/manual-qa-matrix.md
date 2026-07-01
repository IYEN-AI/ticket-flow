# Manual QA Matrix

Date: 2026-07-01

| Surface | Scenario | Command / Evidence | Result |
| --- | --- | --- | --- |
| CLI | Create canonical JSON with `type:ref` source | `OPENCLAW_TICKET_HOME=/tmp/openclaw-ticket-service-qa bun run src/cli.ts create --title 'QA ticket' --type feature --priority high --goal 'prove CLI surface' --acceptance 'ticket json exists' --tag qa,mcp --source discord:999` | Created `T-20260701-001` with active JSON and index entry. |
| CLI | Existing split source form | `OPENCLAW_TICKET_HOME=/tmp/openclaw-ticket-source-qa bun run src/cli.ts create --title 'source split' --source discord 777` | Created source `{ "type": "discord", "ref": "777" }`. |
| CLI | Status/link/log/checkpoint/agent-actions flow | See `docs/VERIFICATION.md` manual CLI QA block. | Status changed to doing, thread linked, log appended, checkpoint current written, agent action listed. |
| CLI | Empty checkpoint guardrail | `OPENCLAW_TICKET_HOME=/tmp/openclaw-ticket-empty-checkpoint bun run src/cli.ts checkpoint T-20260701-001` | Exit 1, message includes `checkpoint requires at least one field`. |
| CLI | Real default store read-only list | `OPENCLAW_TICKET_HOME=/Users/iyen/.openclaw/tickets bun run src/cli.ts list | head -20` | Exit 0, printed real active tickets. |
| Store | Historical data sweep | `TicketSchema.safeParse` over `/Users/iyen/.openclaw/tickets/{active,archive}/**/*.json` | `{"total":346,"failed":0}`. |
| MCP | Happy path stdio | Raw JSON-RPC `initialize`, `tools/list`, `ticket_create`. | JSON-only stdout, ticket created with existing JSON fields. |
| MCP | Agent actions tool | Raw JSON-RPC `ticket_agent_actions`. | Returned `[]` on empty temp store. |
| MCP | Empty checkpoint error path | Raw JSON-RPC `ticket_checkpoint` with only `id`. | Returned `isError: true` with checkpoint guardrail message. |

All automated gates also passed:

```bash
bun run typecheck && bun test && bun run lint
```
