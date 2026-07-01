# OpenClaw Ticket Service Gate Review

## recommendation

REJECT

## originalIntent

Review the new TypeScript/Bun repository for an OpenClaw-compatible ticket CLI and MCP stdio server. The expected outcome is a practical implementation that preserves the existing JSON contract under `~/.openclaw/tickets`, does not invent a new ticket contract, adds CLI and MCP surfaces, keeps MCP stdout safe for JSON-RPC, maintains strict TypeScript hygiene, and provides credible automated/manual verification evidence.

## desiredOutcome

- Existing live and historical OpenClaw ticket JSON remains readable without migration or repair.
- CLI command semantics remain compatible with `/Users/iyen/.openclaw/tickets/ticket.sh`.
- MCP stdio exposes ticket operations without writing application logs to stdout.
- `bun run typecheck`, `bun test`, and `bun run lint` pass.
- Tests and `docs/VERIFICATION.md` cover the compatibility risks, not just new happy-path tickets.
- Source files stay small and responsibilities remain bounded.

## userOutcomeReview

The shipped artifact does not yet satisfy the user's preservation requirement. Against the real default store, `bun run src/cli.ts list` exits non-zero because `src/store/query.ts` reads every active JSON file through a strict `TicketSchema`, and the schema rejects common existing ticket shapes such as missing `tags`, string artifacts, artifacts without `ts`, and non-enum priorities. This means the CLI/MCP implementation can pass temp-store tests while failing for the user's actual `~/.openclaw/tickets` data.

The CLI also accepts an empty `checkpoint` and writes `current: {}` plus an empty checkpoint log entry. The existing shell rejects that command with `Error: checkpoint requires at least one field`, so this is a direct behavior incompatibility.

## blockers

1. High - Real OpenClaw tickets are not backward-compatible with the store schema.
   - Source: `src/store/query.ts:10`, `src/store/io.ts:18`, `src/schema.ts:75`, `src/schema.ts:84`, `src/schema.ts:86`.
   - Evidence: read-only validation of `/Users/iyen/.openclaw/tickets` found 346 ticket files, including 146 active files. 334/346 failed `TicketSchema.safeParse`; sampled failures include `/Users/iyen/.openclaw/tickets/active/T-20260611-009.json` rejected for missing `tags`, `/Users/iyen/.openclaw/tickets/active/T-20260610-008.json` rejected for string artifacts, and `/Users/iyen/.openclaw/tickets/active/T-20260623-001.json` rejected for priority `"2"`.
   - Observable failure: `bun run src/cli.ts list` with the default store exits 1 with `tags: Invalid input: expected array, received undefined`.
   - User impact: the default CLI/MCP read path cannot reliably operate on the preserved ticket store.

2. High - `checkpoint` no-field behavior diverges from `ticket.sh`.
   - Source: `src/schema.ts:142`, `src/store/mutations.ts:54`.
   - Existing contract: `/Users/iyen/.openclaw/tickets/ticket.sh:521` rejects empty checkpoints.
   - Evidence: temp-store probe created a ticket, then ran `bun run src/cli.ts checkpoint <id>` with no fields. It exited 0, printed `<id>: checkpoint logged`, and wrote `"current": {}` with a checkpoint log entry containing only `ts` and `action`.
   - User impact: the CLI can create meaningless checkpoint state that the current OpenClaw shell intentionally forbids.

3. Medium - Verification evidence is too narrow for the compatibility risks.
   - Source: `tests/cli.test.ts:40`, `tests/mcp.test.ts:55`, `docs/VERIFICATION.md:17`, `docs/VERIFICATION.md:46`.
   - Evidence: tests cover newly generated temp-store tickets and an MCP happy path, but there are no fixture tests for representative historical/live tickets, no default-store read-only compatibility sweep, no `done` archive/index scenario, and no empty-checkpoint rejection test.
   - Evidence gap: `.omo/plans/openclaw-ticket-service.md` expected live/minimal fixtures and permissive schema coverage, but no `tests/fixtures/`, `tests/contract.test.ts`, or `tests/schemas.test.ts` artifacts exist in the submitted tree.

4. Medium - Required review artifacts are incomplete for final approval.
   - Evidence gap: no separate code review report, manual QA matrix, executor evidence ledger, or notepad path was provided as input. `docs/VERIFICATION.md` is present, but it does not demonstrate the remove-ai-slops/programming perspective checks, overfit-test checks, or the live-data compatibility class above.

## slopAndOverfitPass

- Loaded and applied `omo:remove-ai-slops` criteria: checked for deletion-only tests, tautological tests, implementation-mirroring tests, excessive tests, unnecessary production abstraction, over-defensive code, dead code, and oversized modules.
- Loaded and applied `omo:programming` TypeScript criteria: strict TS flags, no `any`, no non-null assertions, no default exports, Zod boundary parsing, typed errors, file-size ceiling, and strict test shape.
- Direct findings: no oversized source/test files. Pure LOC counts are all below 250; largest files are `src/mcp.ts` 164 pure LOC, `src/schema.ts` 152 pure LOC, `src/cli.ts` 172 pure LOC.
- Direct concern: the tests are not excessive, but they are underpowered and overfit to newly generated tickets. They do not protect the user's preservation requirement.
- Tool note: the bundled no-excuse checker was attempted but did not run cleanly from the plugin cache due package resolution/path invocation issues; manual criteria were applied directly.

## checkedArtifacts

- `/Users/iyen/projects/openclaw-ticket-service/README.md`
- `/Users/iyen/projects/openclaw-ticket-service/package.json`
- `/Users/iyen/projects/openclaw-ticket-service/tsconfig.json`
- `/Users/iyen/projects/openclaw-ticket-service/biome.jsonc`
- `/Users/iyen/projects/openclaw-ticket-service/docs/VERIFICATION.md`
- `/Users/iyen/projects/openclaw-ticket-service/docs/ULW-RESEARCH-SYNTHESIS.md`
- `/Users/iyen/projects/openclaw-ticket-service/src/cli-support.ts`
- `/Users/iyen/projects/openclaw-ticket-service/src/cli.ts`
- `/Users/iyen/projects/openclaw-ticket-service/src/errors.ts`
- `/Users/iyen/projects/openclaw-ticket-service/src/mcp.ts`
- `/Users/iyen/projects/openclaw-ticket-service/src/schema.ts`
- `/Users/iyen/projects/openclaw-ticket-service/src/store.ts`
- `/Users/iyen/projects/openclaw-ticket-service/src/store/*.ts`
- `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts`
- `/Users/iyen/projects/openclaw-ticket-service/tests/mcp.test.ts`
- `/Users/iyen/projects/openclaw-ticket-service/.omo/plans/openclaw-ticket-service.md`
- `/Users/iyen/.openclaw/tickets/ticket.sh`
- Read-only samples/counts from `/Users/iyen/.openclaw/tickets/active/*.json` and `/Users/iyen/.openclaw/tickets/archive/*/*.json`

## verificationRun

- `bun run typecheck --pretty false`: pass on current tree.
- `bun test`: pass, 3 tests.
- `bun run lint`: pass, Biome checked 19 files.
- MCP stdio raw transcript against temp store: exit 0; three non-empty stdout lines all parsed as JSON; stderr size 0.
- Default-store CLI list: fail, exits 1 due strict schema rejecting existing ticket JSON.
- Empty checkpoint temp-store probe: fail against expected compatibility; command exits 0 and writes an empty checkpoint.

## evidenceGaps

- No committed baseline/diff exists; repository has no commits, so the full tree was reviewed as the submitted artifact.
- No `tests/fixtures/` coverage for live ticket shapes, despite the local plan requiring it.
- No test proving existing archived tickets can be read.
- No test proving `ticket_list`/`tickets://active` tolerates historical active tickets.
- No test for empty checkpoint rejection.
- No code review report artifact showing the requested skill-perspective and slop/overfit coverage before this gate review.
- No manual QA matrix artifact beyond prose in `docs/VERIFICATION.md`.
