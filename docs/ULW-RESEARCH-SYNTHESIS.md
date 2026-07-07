# ULW Research Synthesis: Ticket Flow

Date: 2026-07-01

## Executive Summary

The service is an agent-oriented local ticket system. It should preserve a
simple local ticket contract rather than introduce a remote issue model
immediately. The current contract is file-backed JSON under
`~/.ticket-flow/tickets`, with active tickets in `active/`, closed tickets moved
to `archive/YYYY-MM/`, and `index.json` used as the active-ticket index.

The first implementation therefore keeps local JSON as the source of truth and
adds adapters over the same domain layer: a CLI, an MCP stdio server, and event
projection for `clawhip`. This follows the local-first durability pattern from
tools such as Taskwarrior and todo.txt while avoiding a disruptive
SQLite/workflow migration.

## Research Findings

1. Existing local tickets are heterogeneous. The read contract must tolerate
   older records with optional `current`, nullable or expanded `source`, and
   older log shapes.

2. MCP stdio servers must keep stdout clean. The MCP transport uses stdin/stdout as JSON-RPC and reserves stderr for logs. This implementation avoids application logging in `src/mcp.ts` and verifies stdio with a JSON-RPC transcript.

3. Agile and Kanban sources support the current ticket fields rather than a replacement schema. The important concepts are explicit workflow, WIP visibility, ownership, acceptance criteria, decision history, and bounded handoff packets.

4. Comparable tools split into two useful camps:
   - Taskwarrior and todo.txt show the value of local/offline storage and shell-friendly workflows.
   - GitHub CLI, Linear, Plane, Huly, and Shortcut show richer remote workflow models, but adopting their configurable workflows would be a new contract and was intentionally deferred.

## Implementation Choices

- Keep `T-YYYYMMDD-NNN` IDs.
- Keep `active/*.json`, `archive/YYYY-MM/*.json`, and `index.json`.
- Keep existing statuses: `open`, `doing`, `review`, `blocked`, `done`.
- Keep existing transition guardrail: `review` requires an artifact unless one already exists.
- Keep existing fields: `links`, `log`, `current`, `source`, `acceptance`, `artifacts`.
- Add MCP tools that map directly to existing operations:
  - `ticket_create`
  - `ticket_list`
  - `ticket_get`
  - `ticket_update_status`
  - `ticket_link`
  - `ticket_add_log`
  - `ticket_checkpoint`
  - `ticket_agent_actions`
- Add resource:
  - `tickets://active`

## Sources

- Prior local ticket shell workflow
- MCP TypeScript SDK: https://github.com/modelcontextprotocol/typescript-sdk
- MCP specification: https://modelcontextprotocol.io/specification/2025-06-18
- Agile Manifesto: https://agilemanifesto.org/
- Agile Principles: https://agilemanifesto.org/principles.html
- Scrum Guide: https://scrumguides.org/scrum-guide.html
- Kanban Guide: https://kanbanguides.org/the-kanban-guide/
- Shape Up: https://basecamp.com/shapeup
- Team Topologies concepts: https://teamtopologies.com/key-concepts
- Taskwarrior: https://github.com/GothenburgBitFactory/taskwarrior
- todo.txt-cli: https://github.com/todotxt/todo.txt-cli
- GitHub CLI issue commands: https://cli.github.com/manual/gh_issue

## Gaps

- This repo does not make `clawhip` publishing or thread bootstrapping part of
  the core ticket contract. Those stay in the adapter/integration layer unless
  explicitly productized.
- This repo does not introduce SQLite, UUIDs, configurable workflows, WIP limits, or sync. Research supports them as future options, but they would be new contracts.
