# OpenClaw Ticket Service TypeScript/Bun Plan

## TL;DR
> Summary:      Build the standalone TypeScript/Bun ticket service around the existing `ticket.sh` on-disk JSON contract, with a shared Zod-validated domain/store layer used by both CLI and MCP stdio adapters. Preserve the store layout, IDs, index behavior, status machine, optional side effects, and fixture readability; improve only adapter ergonomics where they do not alter JSON compatibility.
> Deliverables:
> - Bun/TypeScript source for `openclaw-ticket` CLI and MCP stdio server in `/Users/iyen/projects/openclaw-ticket-service`.
> - Zod schemas and shared service/store implementation for the existing ticket JSON contract.
> - Compatibility, unit, CLI, and MCP stdio QA with captured evidence under `.omo/evidence/`.
> - Updated docs/package metadata that explain the preserved contract and `OPENCLAW_TICKET_HOME` override.
> Effort:       Large
> Risk:         Medium - exact compatibility spans shell quirks, historical live data, optional Discord side effects, and a repo that is not currently initialized as Git.

## Scope
### Must have
- Preserve the default store root as `~/.openclaw/tickets`, with `OPENCLAW_TICKET_HOME` as the only default-path override, matching the repository README at `/Users/iyen/projects/openclaw-ticket-service/README.md:5-23`.
- Preserve active/archive/index layout: `active/T-YYYYMMDD-NNN.json`, `archive/YYYY-MM/T-YYYYMMDD-NNN.json`, and `index.json`, matching `/Users/iyen/.openclaw/tickets/ticket.sh:4-12` and archive behavior at `/Users/iyen/.openclaw/tickets/ticket.sh:276-287`.
- Preserve ticket JSON fields created by the shell: `id`, `title`, `status`, `priority`, `type`, `source`, `goal`, `acceptance`, `tags`, `artifacts`, `links`, `parent`, `children`, `assignee`, `created`, `updated`, `closed`, `log`, matching `/Users/iyen/.openclaw/tickets/ticket.sh:168-191`.
- Preserve optional live fields `current` and `next_action`, including sparse checkpoint entries, matching `/Users/iyen/.openclaw/tickets/ticket.sh:530-559` and samples at `/Users/iyen/.openclaw/tickets/active/T-20260616-001.json:179-188`.
- Preserve the index shape `{ version, lastId, tickets }`, and each ticket summary shape `{ title, status, priority, type, updated }`, matching `/Users/iyen/.openclaw/tickets/index.json:1-10` and update logic at `/Users/iyen/.openclaw/tickets/ticket.sh:78-90`.
- Preserve command names/options for `create`, `status`, `list`, `agent-actions`, `show`, `link`, `log`, `checkpoint`, and `help`, matching `/Users/iyen/.openclaw/tickets/ticket.sh:566-587`.
- Preserve the status transition graph and review artifact guard from `/Users/iyen/.openclaw/tickets/ticket.sh:248-262`.
- Preserve `done` semantics: set `closed`, move to `archive/YYYY-MM/`, delete active JSON, and remove the ticket from `index.json`, matching `/Users/iyen/.openclaw/tickets/ticket.sh:276-310`.
- Preserve optional side effects: `TICKET_THREAD_BOOTSTRAP=1`, `TICKET_PUBLISH_PARENT=1`, best-effort `clawhip send`, and status publish for `review|done|blocked`, matching `/Users/iyen/.openclaw/tickets/ticket.sh:16-47` and `/Users/iyen/.openclaw/tickets/ticket.sh:207-217`.
- Use strict TypeScript, the current repo's Biome lint setup, and Zod at all storage, CLI, and MCP boundaries, matching `/Users/iyen/projects/openclaw-ticket-service/tsconfig.json:1-23` and `/Users/iyen/projects/openclaw-ticket-service/biome.jsonc:1-45`.
- Use stable `@modelcontextprotocol/sdk@1.29.0` import paths (`@modelcontextprotocol/sdk/server/mcp.js`, `@modelcontextprotocol/sdk/server/stdio.js`, client stdio for QA), based on the stable docs at `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/server.md` and `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/client.md`.
- Expose MCP stdio tools for every non-help ticket operation: `ticket_create`, `ticket_status`, `ticket_list`, `ticket_show`, `ticket_link`, `ticket_log`, `ticket_checkpoint`, and `ticket_agent_actions`.
- Tests first: each implementation task starts by adding or expanding failing Bun tests before production source changes.

### Must NOT have (guardrails, anti-slop, scope boundaries)
- Must not replace the JSON file store with SQLite, a service daemon, a DB, or a different schema.
- Must not tighten existing permissive fields such as `priority`, `type`, `source.type`, or tags into new enums; only preserve existing validations for statuses and `next_action.type`.
- Must not mutate the real `~/.openclaw/tickets` store in automated tests or QA; every write scenario must use a temp `OPENCLAW_TICKET_HOME`.
- Must not clean up or rewrite historical dirty live index entries unless a later explicit repair command is planned; new `done` transitions must follow `ticket.sh` and remove the entry.
- Must not depend on beta `@modelcontextprotocol/server` or `@modelcontextprotocol/client` packages for this release.
- Must not add HTTP/SSE MCP transport, web UI, daemon mode, auth, sync, or migration tooling.
- Must not send real Discord/clawhip messages in tests; side effects must be dependency-injected or PATH-shadowed with local fakes.
- Must not duplicate ticket business logic separately in CLI and MCP adapters; adapters must call the shared service layer.
- Must not reproduce shell implementation bugs that are not JSON contract, such as duplicate create stdout. CLI create should print one ticket id line while JSON compatibility remains exact.

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: TDD + Bun test (`bun test`) with strict compiler (`bun run typecheck`) and Biome (`bun run lint`).
- QA policy: every task has agent-executed scenarios.
- Evidence: `.omo/evidence/task-<N>-<slug>.<ext>`
- Real CLI QA: execute `bun run cli ...` with a temp `OPENCLAW_TICKET_HOME`, assert files and stdout/stderr.
- Real MCP QA: execute a Bun MCP client over stdio, not a mocked function call, using the SDK client transport or raw JSON-RPC where specified.
- Contract QA: compare generated temp-store JSON against contract fixtures derived from `/Users/iyen/.openclaw/tickets/ticket.sh` behavior and live sample fixtures.

## Execution strategy
### Parallel execution waves
> Target 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks to maximize parallelism.

Wave 1 (no dependencies):
- Task 1: Baseline repo, Git, and toolchain guardrails
- Task 2: Contract fixtures and test helpers
- Task 3: Zod schemas and exported TypeScript contract types
- Task 4: Store path, index initialization, and safe file primitives
- Task 5: Real QA scripts and evidence harness

Wave 2 (after Wave 1):
- Task 6: Store engine, ID generation, archive movement, and parent-child persistence; depends [2, 3, 4]
- Task 7: Core ticket service for create/list/show/status; depends [3, 4, 6]
- Task 8: Extended service operations for link/log/checkpoint/agent-actions; depends [3, 4, 6]
- Task 9: Optional side-effect adapter for clawhip/thread bootstrap; depends [4, 6, 7]
- Task 10: CLI adapter with full command parity; depends [5, 7, 8, 9]

Wave 3 (after Wave 2):
- Task 11: MCP stdio adapter and SDK client QA; depends [5, 7, 8, 9]
- Task 12: Documentation, package surface, and dependency lock finalization; depends [10, 11]
- Task 13: End-to-end compatibility sweep and fixture regression suite; depends [10, 11, 12]

Critical path: Task 1 -> Task 3 -> Task 6 -> Task 7 -> Task 10 -> Task 13

### Dependency matrix
| Task | Depends on | Blocks | Can parallelize with |
|------|------------|--------|----------------------|
| 1    | none       | 6, 10, 12, 13 | 2, 3, 4, 5 |
| 2    | none       | 6, 7, 8, 13 | 1, 3, 4, 5 |
| 3    | none       | 6, 7, 8, 10, 11 | 1, 2, 4, 5 |
| 4    | none       | 6, 7, 8, 9 | 1, 2, 3, 5 |
| 5    | none       | 10, 11, 13 | 1, 2, 3, 4 |
| 6    | 2, 3, 4   | 7, 8, 9, 13 | none in Wave 2 until merged |
| 7    | 3, 4, 6   | 9, 10, 11, 13 | 8 |
| 8    | 3, 4, 6   | 10, 11, 13 | 7, 9 |
| 9    | 4, 6, 7   | 10, 11, 13 | 8 |
| 10   | 5, 7, 8, 9 | 12, 13 | 11 |
| 11   | 5, 7, 8, 9 | 12, 13 | 10 |
| 12   | 10, 11    | 13 | none |
| 13   | 10, 11, 12 | final verification | none |

## Todos
> Implementation + Test = ONE task. Never separate.
> Every task MUST have: References + Acceptance Criteria + QA Scenarios + Commit.

- [ ] 1. Baseline repo, Git, and toolchain guardrails

  What to do: Initialize Git if still absent; record the current scaffold as the baseline; verify `package.json`, `README.md`, `tsconfig.json`, `biome.jsonc`, `bun.lock`, and existing tests are preserved. If `.git` already exists by execution time, do not reinitialize; inspect status and commit only the current task's changes. Add `packageManager` only if Bun requires it for reproducibility, otherwise leave package metadata unchanged.
  Must NOT do: Do not create source implementation in this task. Do not update dependencies opportunistically. Do not delete current tests.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [6, 10, 12, 13] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/package.json:1` - existing Bun-first manifest and script names.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/tsconfig.json:1` - strict TS settings to preserve.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/biome.jsonc:1` - lint/format rules to preserve.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:1` - existing failing-first CLI test file.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/tests/mcp.test.ts:1` - existing failing-first MCP test file.
  - External: `https://github.com/oven-sh/bun/blob/main/README.md` - Bun `bun test`, `bun run`, and package management commands.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && git status --short --branch` exits 0.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && test -f package.json -a -f README.md -a -f tsconfig.json -a -f biome.jsonc -a -f bun.lock`.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && test ! -f src/cli.ts -a ! -f src/mcp.ts` before later implementation tasks begin.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun --version && bun install --frozen-lockfile` succeeds without changing `bun.lock`.

  QA scenarios (MANDATORY - task incomplete without these):
  > Name the exact tool AND its exact invocation - not "verify it works". Browser use: in Codex, use `browser:control-in-app-browser` first when available and no authenticated/persistent user browser profile is required; otherwise use Chrome to drive the page, or agent-browser (https://github.com/vercel-labs/agent-browser) when Chrome is unavailable. Computer use: OS-level GUI automation for a non-browser desktop app.
  ```
  Scenario: baseline inventory
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && { git status --short --branch; find . -maxdepth 2 -type f | sort; } > .omo/evidence/task-1-baseline.txt
    Expected: Evidence includes package/config/tests/lock files and no src/*.ts implementation files.
    Evidence: .omo/evidence/task-1-baseline.txt

  Scenario: frozen install rejects drift
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun install --frozen-lockfile > .omo/evidence/task-1-install.txt 2>&1
    Expected: Command exits 0 and `git diff -- bun.lock package.json` is empty.
    Evidence: .omo/evidence/task-1-install.txt
  ```

  Commit: YES | Message: `chore(repo): establish ticket service baseline` | Files: [`package.json`, `README.md`, `tsconfig.json`, `biome.jsonc`, `.gitignore`, `bun.lock`, `tests/cli.test.ts`, `tests/mcp.test.ts`]

- [ ] 2. Contract fixtures and test helpers

  What to do: Add fixture JSON and helpers that encode the current shell contract. Create `tests/fixtures/live-ticket-current.json`, `tests/fixtures/live-ticket-minimal.json`, `tests/fixtures/index.sample.json`, and `tests/helpers/store.ts`. Expand tests first to assert fixture parseability, JSON pretty-printing with 2 spaces, `ensure_ascii=false` equivalent Unicode preservation, and temp-store isolation.
  Must NOT do: Do not call the real `~/.openclaw/tickets` store from tests except read-only fixture copying at development time. Do not snapshot secrets or Discord tokens.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [6, 7, 8, 13] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:168` - create-time ticket field order and defaults.
  - Pattern:  `/Users/iyen/.openclaw/tickets/index.json:1` - index top-level shape.
  - Pattern:  `/Users/iyen/.openclaw/tickets/active/T-20260616-001.json:43` - rich `log`, `artifacts`, `links`, and `current` sample.
  - Pattern:  `/Users/iyen/.openclaw/tickets/active/T-20260623-001.json:1` - minimal/permissive live sample with priority `"2"` and `type: "patch"`.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:30` - temp store setup pattern to improve.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/contract.test.ts` passes.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && test -f tests/fixtures/live-ticket-current.json -a -f tests/fixtures/live-ticket-minimal.json -a -f tests/fixtures/index.sample.json`.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && rg 'openclaw/tickets' tests/fixtures tests/helpers` finds only fixture comments or paths, not writes to the real store.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: fixture compatibility parse
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/contract.test.ts --reporter=junit > .omo/evidence/task-2-contract.xml
    Expected: JUnit evidence shows fixture parse tests pass.
    Evidence: .omo/evidence/task-2-contract.xml

  Scenario: temp-store isolation
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && HOME_BEFORE="$HOME" bun test tests/helpers/store.test.ts > .omo/evidence/task-2-store-helper.txt 2>&1
    Expected: Test creates and removes only a temporary directory and does not modify `$HOME/.openclaw/tickets`.
    Evidence: .omo/evidence/task-2-store-helper.txt
  ```

  Commit: YES | Message: `test(contract): capture existing ticket JSON fixtures` | Files: [`tests/contract.test.ts`, `tests/helpers/store.ts`, `tests/helpers/store.test.ts`, `tests/fixtures/live-ticket-current.json`, `tests/fixtures/live-ticket-minimal.json`, `tests/fixtures/index.sample.json`]

- [ ] 3. Zod schemas and exported TypeScript contract types

  What to do: Add `src/domain/schemas.ts` and `src/domain/types.ts`. Use Zod to validate ticket files, index files, CLI/MCP inputs, artifacts, links, checkpoint entries, and `next_action`. Keep `priority`, `type`, `source.type`, and most text fields permissive strings. Enforce only known statuses (`open`, `doing`, `review`, `blocked`, `done`) and checkpoint `next_action.type` (`agent_action`, `owner_gate`, `release_gate`, `blocked`), matching the shell.
  Must NOT do: Do not invent new required fields. Do not reject historical tickets that omit `current`, use numeric-looking priority strings, contain Korean text, or contain sparse `next_action`.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [6, 7, 8, 10, 11] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - API/Type: `/Users/iyen/.openclaw/tickets/ticket.sh:170` - ticket fields to model.
  - API/Type: `/Users/iyen/.openclaw/tickets/ticket.sh:248` - status transition statuses.
  - API/Type: `/Users/iyen/.openclaw/tickets/ticket.sh:525` - `next_action.type` allowed values.
  - Pattern:  `/Users/iyen/.openclaw/tickets/active/T-20260623-001.json:5` - priority can be non-enum string.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/tsconfig.json:8` - strict TypeScript requirement.
  - External: `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/protocol.md` - SDK uses Zod schemas for tools.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/schemas.test.ts` passes.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun run typecheck` passes after schema files are added.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun run lint` passes with no `any`, default export, non-null assertion, or unused code violations.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: rich fixture validates
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/schemas.test.ts -t "rich live ticket fixture validates" > .omo/evidence/task-3-schema-rich.txt 2>&1
    Expected: Test exits 0 and validates `current.next_action` plus sparse log entries.
    Evidence: .omo/evidence/task-3-schema-rich.txt

  Scenario: invalid next_action rejected
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/schemas.test.ts -t "invalid next action type is rejected" > .omo/evidence/task-3-schema-error.txt 2>&1
    Expected: Test exits 0 and assertion confirms Zod rejects the invalid type.
    Evidence: .omo/evidence/task-3-schema-error.txt
  ```

  Commit: YES | Message: `feat(contract): add zod ticket schemas` | Files: [`src/domain/schemas.ts`, `src/domain/types.ts`, `tests/schemas.test.ts`]

- [ ] 4. Store path, index initialization, and safe file primitives

  What to do: Add `src/domain/paths.ts`, `src/domain/json.ts`, and `src/domain/index-store.ts`. Implement path resolution with `OPENCLAW_TICKET_HOME` override, default `join(homedir(), ".openclaw", "tickets")`, `active/` and `archive/` creation, index loading, index creation when absent, and atomic JSON writes via temp file + rename. Preserve existing index fields when present.
  Must NOT do: Do not change the default path to the repo directory. Do not require `index.json` to already exist in new temp stores. Do not write outside the resolved store root.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [6, 7, 8, 9] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:4` - default shell store path.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:11` - active/archive directory creation.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:78` - index update contract.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:12` - tests pass `OPENCLAW_TICKET_HOME`.
  - External: `https://bun.sh/docs/runtime/file-io` - Bun file I/O behavior.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/paths.test.ts tests/index-store.test.ts` passes.
  - [ ] Missing `index.json` in a temp store is created as `{ "version": 1, "lastId": null, "tickets": {} }` without touching real home.
  - [ ] Existing `index.json` with extra top-level keys is preserved except for intended ticket mutations.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: default path resolves without writing
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/paths.test.ts -t "default ticket home resolves to user openclaw tickets" > .omo/evidence/task-4-default-path.txt 2>&1
    Expected: Test asserts the path string equals `$HOME/.openclaw/tickets` and performs no writes.
    Evidence: .omo/evidence/task-4-default-path.txt

  Scenario: missing index repaired in temp store
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/index-store.test.ts -t "creates missing index in temp store" > .omo/evidence/task-4-index-create.txt 2>&1
    Expected: Test exits 0 and temp `index.json` has `version`, `lastId`, and empty `tickets`.
    Evidence: .omo/evidence/task-4-index-create.txt
  ```

  Commit: YES | Message: `feat(store): resolve ticket home and initialize index` | Files: [`src/domain/paths.ts`, `src/domain/json.ts`, `src/domain/index-store.ts`, `tests/paths.test.ts`, `tests/index-store.test.ts`]

- [ ] 5. Real QA scripts and evidence harness

  What to do: Add executable QA helpers under `scripts/`: `scripts/cli-smoke.ts`, `scripts/mcp-smoke.ts`, and `scripts/write-evidence.ts` if useful. These scripts must create temp stores, run real CLI/MCP processes, assert observable output/files, and write concise evidence. Add package scripts `qa:cli`, `qa:mcp`, and `qa:all` only if they do not conflict with existing scripts.
  Must NOT do: Do not make QA scripts import private test-only internals to bypass real process execution. Do not write evidence outside `.omo/evidence/`.

  Parallelization: Can parallel: YES | Wave 1 | Blocks: [10, 11, 13] | Blocked by: []

  References (executor has NO interview context - be exhaustive):
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:12` - current process-spawn pattern for CLI.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/mcp.test.ts:13` - current raw stdio MCP test pattern.
  - External: `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/client.md` - stdio client transport for real MCP QA.
  - External: `https://github.com/oven-sh/bun/blob/main/README.md` - `bun run` and `bun test` usage.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun run scripts/cli-smoke.ts --help` exits 0 and explains required env/commands.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun run scripts/mcp-smoke.ts --help` exits 0 and explains its stdio flow.
  - [ ] Scripts fail clearly before implementation with a missing entrypoint error, then pass after Tasks 10 and 11.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: QA script help
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && { bun run scripts/cli-smoke.ts --help; bun run scripts/mcp-smoke.ts --help; } > .omo/evidence/task-5-help.txt 2>&1
    Expected: Evidence contains usage for both smoke scripts and exits 0.
    Evidence: .omo/evidence/task-5-help.txt

  Scenario: missing entrypoint fails clearly
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun run scripts/cli-smoke.ts > .omo/evidence/task-5-preimpl-error.txt 2>&1; test $? -ne 0
    Expected: Evidence mentions missing `src/cli.ts` or command failure before CLI implementation.
    Evidence: .omo/evidence/task-5-preimpl-error.txt
  ```

  Commit: YES | Message: `test(qa): add real process smoke harnesses` | Files: [`scripts/cli-smoke.ts`, `scripts/mcp-smoke.ts`, `scripts/write-evidence.ts`, `package.json`]

- [ ] 6. Store engine, ID generation, archive movement, and parent-child persistence

  What to do: Add `src/domain/store.ts` with create/read/update/move primitives. Implement `_next_id` equivalent by scanning index keys, active files, and archived month folders for today's UTC `T-YYYYMMDD-NNN` ids. Add best-effort create locking (`.openclaw-ticket.lock` under the resolved store root, exclusive create with timeout, cleanup on success/failure) to improve concurrent create without changing JSON contract. Implement parent-child update only when active parent file exists.
  Must NOT do: Do not use local time for IDs/timestamps; shell uses UTC at `/Users/iyen/.openclaw/tickets/ticket.sh:52-53`. Do not fail create when parent is missing; shell silently skips parent mutation if the active parent file is absent.

  Parallelization: Can parallel: NO | Wave 2 | Blocks: [7, 8, 9, 13] | Blocked by: [2, 3, 4]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:52` - UTC date/time helpers.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:55` - next id scans index, active, and archive.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:194` - parent child append only for active parent.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:276` - archive move behavior.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/contract.test.ts` - fixture expectations from Task 2.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/store.test.ts` passes.
  - [ ] Store test creates `T-<UTC today>-001`, then `-002`, including when the first id exists only under `archive/YYYY-MM/`.
  - [ ] Store test verifies `done` move removes active file and writes archived file under the UTC month.
  - [ ] Store test verifies active parent `children` append and missing/archived parent no-op.
  - [ ] Concurrent create test with at least 10 parallel creates produces unique ids and no stale lock.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: archive id scan
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/store.test.ts -t "next id scans archive and active files" > .omo/evidence/task-6-id-scan.txt 2>&1
    Expected: Test exits 0 and asserts the next id increments past archived tickets.
    Evidence: .omo/evidence/task-6-id-scan.txt

  Scenario: concurrent create lock
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/store.test.ts -t "parallel creates produce unique ids" > .omo/evidence/task-6-concurrent.txt 2>&1
    Expected: Test exits 0, creates at least 10 unique ids, and no lock file remains.
    Evidence: .omo/evidence/task-6-concurrent.txt
  ```

  Commit: YES | Message: `feat(store): preserve ticket file lifecycle` | Files: [`src/domain/store.ts`, `tests/store.test.ts`]

- [ ] 7. Core ticket service for create/list/show/status

  What to do: Add `src/domain/service.ts` for create, list, show, and status transitions. Implement create defaults, source parsing, acceptance/tag parsing, artifacts/log initialization, index update, list sorting/filtering, show active-or-archive lookup, status transition validation, review artifact guard, artifact/evidence append, closed timestamp, and done index removal.
  Must NOT do: Do not make `priority` or `type` enums. Do not let `status`, `link`, `log`, or `checkpoint` mutate archived tickets. Do not publish side effects directly here; call a side-effect port from Task 9.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [9, 10, 11, 13] | Blocked by: [3, 4, 6]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:106` - create parser defaults and options.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:144` - acceptance/tag/source conversion.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:218` - status command inputs.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:248` - transition graph.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:320` - list reads index and filters status.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:386` - show searches archive only if not active.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:40` - existing create contract expectation.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/service-core.test.ts tests/cli.test.ts` passes for domain-level create/status/list/show expectations.
  - [ ] Review transition without artifact fails with `Error: review requires --artifact`.
  - [ ] Invalid transition fails with `Error: cannot transition <old> → <new>`.
  - [ ] `show` reads archived done ticket, while `status` on archived id returns `Error: ticket <id> not found`.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: create and list contract
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/service-core.test.ts -t "create writes ticket and index contract" > .omo/evidence/task-7-create-list.txt 2>&1
    Expected: Test exits 0 and asserts ticket fields plus index summary.
    Evidence: .omo/evidence/task-7-create-list.txt

  Scenario: invalid status transition
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/service-core.test.ts -t "invalid status transition returns shell-compatible error" > .omo/evidence/task-7-transition-error.txt 2>&1
    Expected: Test exits 0 and confirms the exact error string.
    Evidence: .omo/evidence/task-7-transition-error.txt
  ```

  Commit: YES | Message: `feat(service): implement core ticket operations` | Files: [`src/domain/service.ts`, `tests/service-core.test.ts`, `tests/cli.test.ts`]

- [ ] 8. Extended service operations for link/log/checkpoint/agent-actions

  What to do: Extend `src/domain/service.ts` or add `src/domain/checkpoint.ts` for `link`, `log`, `checkpoint`, and `agent-actions`. Implement link de-duplication for `github_issues`, `prs`, `threads`, `cron_jobs`; log note append; checkpoint sparse entry/current update; `next_action` validation; `agent-actions --stale-minutes` scan that skips malformed ticket JSON like the shell.
  Must NOT do: Do not require all checkpoint fields. Do not reject a checkpoint that only contains `next_action`. Do not print malformed ticket errors from `agent-actions`; shell skips them.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [10, 11, 13] | Blocked by: [3, 4, 6]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:347` - `agent-actions` implementation and stale filtering.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:447` - link command and allowed link buckets.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:477` - log command behavior.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:496` - checkpoint options.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:521` - checkpoint requires at least one field.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:525` - checkpoint next type validation.
  - Pattern:  `/Users/iyen/.openclaw/tickets/active/T-20260623-001.json:85` - live `current` shape with sparse `next_action`.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/service-extended.test.ts` passes.
  - [ ] Link operation is idempotent and updates `updated` only when a new link is added.
  - [ ] Empty checkpoint fails with `Error: checkpoint requires at least one field`.
  - [ ] Invalid next type fails with `Error: --next-type must be one of agent_action, owner_gate, release_gate, blocked`.
  - [ ] `agent-actions --stale-minutes 15` includes only stale `current.next_action.type=agent_action` tickets.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: checkpoint current update
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/service-extended.test.ts -t "checkpoint persists current next_action" > .omo/evidence/task-8-checkpoint.txt 2>&1
    Expected: Test exits 0 and asserts both log entry and current object are updated.
    Evidence: .omo/evidence/task-8-checkpoint.txt

  Scenario: malformed ticket skipped by agent-actions
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/service-extended.test.ts -t "agent actions skips malformed ticket json" > .omo/evidence/task-8-agent-actions-error.txt 2>&1
    Expected: Test exits 0 and confirms no stderr/error for malformed JSON.
    Evidence: .omo/evidence/task-8-agent-actions-error.txt
  ```

  Commit: YES | Message: `feat(service): implement checkpoint and link operations` | Files: [`src/domain/service.ts`, `src/domain/checkpoint.ts`, `tests/service-extended.test.ts`]

- [ ] 9. Optional side-effect adapter for clawhip/thread bootstrap

  What to do: Add `src/adapters/side-effects.ts` and tests that preserve best-effort behavior. Implement priority emoji selection, create publish when `TICKET_PUBLISH_PARENT=1`, status publish for `review|done|blocked`, and thread bootstrap when `TICKET_THREAD_BOOTSTRAP=1` and `<ticketHome>/thread-dispatcher.mjs` is executable. Use injected process runner in unit tests and PATH-shadowed fake `clawhip` in QA.
  Must NOT do: Do not call real `clawhip` or Discord during tests. Do not fail ticket operations when side-effect commands fail. Do not create Discord threads by default.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [10, 11, 13] | Blocked by: [4, 6, 7]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:16` - `_publish_ticket` message shape and emoji mapping.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:33` - `_publish_status_change` message shape.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:207` - create side-effect routing and env vars.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:312` - status publish trigger states.
  - Pattern:  `/Users/iyen/.openclaw/tickets/thread-dispatcher.mjs:25` - dispatcher no-op if thread already linked.
  - Pattern:  `/Users/iyen/.openclaw/tickets/thread-dispatcher.mjs:138` - dispatcher appends thread link/log.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/side-effects.test.ts` passes.
  - [ ] Side-effect test proves no command runs when env vars are unset.
  - [ ] Side-effect test proves failed `clawhip` exits do not fail create/status.
  - [ ] Thread bootstrap test uses a fake executable `thread-dispatcher.mjs` under temp store and verifies invocation only when `TICKET_THREAD_BOOTSTRAP=1`.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: fake clawhip publish
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/side-effects.test.ts -t "publishes create through fake clawhip only when enabled" > .omo/evidence/task-9-publish.txt 2>&1
    Expected: Test exits 0 and fake clawhip capture contains the expected channel/message.
    Evidence: .omo/evidence/task-9-publish.txt

  Scenario: side-effect failure is non-fatal
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/side-effects.test.ts -t "clawhip failure is best effort" > .omo/evidence/task-9-side-effect-error.txt 2>&1
    Expected: Test exits 0 and ticket operation still succeeds despite fake command failure.
    Evidence: .omo/evidence/task-9-side-effect-error.txt
  ```

  Commit: YES | Message: `feat(adapters): preserve optional ticket side effects` | Files: [`src/adapters/side-effects.ts`, `tests/side-effects.test.ts`]

- [ ] 10. CLI adapter with full command parity

  What to do: Implement `src/cli.ts` as a thin adapter over the shared service. Preserve command names and options from the shell. Configure parsing so unknown options, missing required fields, not-found errors, transition errors, and checkpoint validation errors are shell-compatible. Keep improved `create` stdout to a single id line and document this as adapter cleanup, not JSON contract change. Expand existing `tests/cli.test.ts`.
  Must NOT do: Do not duplicate service logic in the CLI. Do not write to real home in tests. Do not let command parser default errors leak as non-contract Commander wording where shell has explicit wording.

  Parallelization: Can parallel: YES | Wave 2 | Blocks: [12, 13] | Blocked by: [5, 7, 8, 9]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:112` - create option parsing.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:223` - status option parsing.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:320` - list output.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:399` - show output fields.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:447` - link flags.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:566` - dispatch/help text.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:12` - real Bun process spawn helper.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/cli.test.ts` passes.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun run typecheck && bun run lint` passes.
  - [ ] CLI covers all commands: `create`, `status`, `list`, `agent-actions`, `show`, `link`, `log`, `checkpoint`, `help`.
  - [ ] `OPENCLAW_TICKET_HOME=$(mktemp -d) bun run cli create --title "QA"` prints exactly one id line matching `^T-[0-9]{8}-001$`.
  - [ ] `bun run cli create` exits 1 and stderr contains `Error: --title required`.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: real CLI happy path
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && STORE="$(mktemp -d)" && OPENCLAW_TICKET_HOME="$STORE" bun run cli create --title "CLI QA" --type feature --priority high --goal "real qa" --acceptance "json exists" --tag "mcp,cli" --source discord:123 > .omo/evidence/task-10-cli-create.txt 2> .omo/evidence/task-10-cli-create.err && ID="$(cat .omo/evidence/task-10-cli-create.txt)" && test -f "$STORE/active/$ID.json" && OPENCLAW_TICKET_HOME="$STORE" bun run cli list > .omo/evidence/task-10-cli-list.txt
    Expected: Create exits 0, stderr is empty, active JSON exists, and list evidence contains `[open] [high] CLI QA`.
    Evidence: .omo/evidence/task-10-cli-create.txt

  Scenario: real CLI review guard
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && STORE="$(mktemp -d)" && ID="$(OPENCLAW_TICKET_HOME="$STORE" bun run cli create --title "Review guard")" && OPENCLAW_TICKET_HOME="$STORE" bun run cli status "$ID" doing >/dev/null && OPENCLAW_TICKET_HOME="$STORE" bun run cli status "$ID" review > .omo/evidence/task-10-cli-review-guard.txt 2>&1; test $? -ne 0
    Expected: Evidence contains `Error: review requires --artifact`.
    Evidence: .omo/evidence/task-10-cli-review-guard.txt
  ```

  Commit: YES | Message: `feat(cli): add command-compatible ticket adapter` | Files: [`src/cli.ts`, `tests/cli.test.ts`]

- [ ] 11. MCP stdio adapter and SDK client QA

  What to do: Implement `src/mcp.ts` and, if useful, `src/mcp/server.ts`. Register all ticket operation tools with Zod input schemas and structured outputs. Use stable SDK v1.29 imports. Ensure readiness/log messages go to stderr only; stdout must be reserved for JSON-RPC. Expand `tests/mcp.test.ts` to use SDK client stdio where possible and raw JSON-RPC only for protocol edge coverage.
  Must NOT do: Do not use beta split packages. Do not `console.log` server readiness to stdout. Do not expose a partial MCP surface limited to `ticket_create`.

  Parallelization: Can parallel: YES | Wave 3 | Blocks: [12, 13] | Blocked by: [5, 7, 8, 9]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/tests/mcp.test.ts:13` - current raw stdio test helper.
  - External: `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/server.md` - `McpServer` and `StdioServerTransport`.
  - External: `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/client.md` - `StdioClientTransport` for QA.
  - External: `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/protocol.md` - tool registration with Zod schemas.
  - External: `https://bun.sh/docs/guides/write-file/stdout` - stdout behavior; MCP protocol must own stdout.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/mcp.test.ts` passes.
  - [ ] `tools/list` returns all eight tools: `ticket_create`, `ticket_status`, `ticket_list`, `ticket_show`, `ticket_link`, `ticket_log`, `ticket_checkpoint`, `ticket_agent_actions`.
  - [ ] `ticket_create` and `ticket_status` calls mutate the same temp store as CLI.
  - [ ] Invalid MCP input returns an MCP tool error and does not write a ticket file.
  - [ ] MCP server emits no non-JSON protocol text on stdout.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: real MCP create/list/status flow
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && OPENCLAW_TICKET_HOME="$(mktemp -d)" bun run scripts/mcp-smoke.ts > .omo/evidence/task-11-mcp-smoke.json 2> .omo/evidence/task-11-mcp-smoke.err
    Expected: Smoke exits 0, JSON evidence contains created ticket id plus tool list, and stderr contains only diagnostic text.
    Evidence: .omo/evidence/task-11-mcp-smoke.json

  Scenario: real MCP invalid input
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && OPENCLAW_TICKET_HOME="$(mktemp -d)" bun run scripts/mcp-smoke.ts --invalid-input > .omo/evidence/task-11-mcp-invalid.json 2> .omo/evidence/task-11-mcp-invalid.err
    Expected: Smoke exits 0 and evidence shows an MCP error result for missing/invalid title with no active ticket file.
    Evidence: .omo/evidence/task-11-mcp-invalid.json
  ```

  Commit: YES | Message: `feat(mcp): expose ticket tools over stdio` | Files: [`src/mcp.ts`, `src/mcp/server.ts`, `tests/mcp.test.ts`, `scripts/mcp-smoke.ts`]

- [ ] 12. Documentation, package surface, and dependency lock finalization

  What to do: Update README and package metadata to match the implemented CLI/MCP surface. Document the exact preserved JSON contract, env vars, command matrix, MCP tools, temp-store test safety, and known compatibility decisions. Verify `bun.lock` still pins stable SDK v1.29.0 and does not pull beta split packages. If Git was initialized in Task 1, commit docs/package changes separately.
  Must NOT do: Do not add install instructions that point at unimplemented commands. Do not claim real Discord sends were tested. Do not publish or tag a package.

  Parallelization: Can parallel: NO | Wave 3 | Blocks: [13] | Blocked by: [10, 11]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/README.md:1` - current README.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/package.json:6` - CLI bin entry.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/package.json:16` - current dependency list.
  - Pattern:  `/Users/iyen/projects/openclaw-ticket-service/bun.lock:1` - lockfile format and resolved versions.
  - External: `https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/docs/server.md` - MCP import/runtime notes to cite.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun install --frozen-lockfile && bun run typecheck && bun run lint && bun test` passes.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && rg '@modelcontextprotocol/(server|client)' package.json bun.lock src tests` returns no matches.
  - [ ] README documents all CLI commands and all eight MCP tools.
  - [ ] README states default store and `OPENCLAW_TICKET_HOME` override exactly.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: dependency surface audit
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && { bun install --frozen-lockfile; rg '@modelcontextprotocol/(server|client)' package.json bun.lock src tests || true; rg '@modelcontextprotocol/sdk@1.29.0' bun.lock; } > .omo/evidence/task-12-deps.txt 2>&1
    Expected: Evidence shows frozen install success, stable SDK lock entry, and no beta split package imports.
    Evidence: .omo/evidence/task-12-deps.txt

  Scenario: README command coverage
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && for term in create status list agent-actions show link log checkpoint ticket_create ticket_status ticket_list ticket_show ticket_link ticket_log ticket_checkpoint ticket_agent_actions OPENCLAW_TICKET_HOME; do rg "$term" README.md >/dev/null || exit 1; done; printf 'README coverage ok\n' > .omo/evidence/task-12-readme.txt
    Expected: Evidence says README coverage ok.
    Evidence: .omo/evidence/task-12-readme.txt
  ```

  Commit: YES | Message: `docs(readme): document ticket cli and mcp contract` | Files: [`README.md`, `package.json`, `bun.lock`]

- [ ] 13. End-to-end compatibility sweep and fixture regression suite

  What to do: Add or finalize `tests/e2e.compat.test.ts` that exercises the full shell command lifecycle through the CLI and then confirms MCP can read/mutate the same temp store. Cover create with parent, source forms, status to done, archive show, link/log/checkpoint/current, agent-actions stale filter, side-effect fakes, list status filters, unknown option errors, and malformed JSON skip. Run full QA scripts and capture evidence.
  Must NOT do: Do not use mocks for CLI/MCP process execution in this sweep. Do not write to the real store. Do not skip failure scenarios just because unit tests cover them.

  Parallelization: Can parallel: NO | Wave 3 | Blocks: [final verification] | Blocked by: [10, 11, 12]

  References (executor has NO interview context - be exhaustive):
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:566` - complete shell command matrix.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:248` - transition graph for lifecycle sweep.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:306` - done removes from index.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:347` - agent-actions scan behavior.
  - Pattern:  `/Users/iyen/.openclaw/tickets/ticket.sh:586` - unknown command error.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/cli.test.ts:40` - existing CLI seed scenario.
  - Test:     `/Users/iyen/projects/openclaw-ticket-service/tests/mcp.test.ts:52` - existing MCP seed scenario.

  Acceptance criteria (agent-executable only):
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/e2e.compat.test.ts` passes.
  - [ ] `cd /Users/iyen/projects/openclaw-ticket-service && bun run qa:all` passes if `qa:all` exists; otherwise `bun run scripts/cli-smoke.ts && bun run scripts/mcp-smoke.ts` passes.
  - [ ] Full suite passes: `cd /Users/iyen/projects/openclaw-ticket-service && bun install --frozen-lockfile && bun run typecheck && bun run lint && bun test`.
  - [ ] Evidence files exist for every task and final e2e run: `find .omo/evidence -type f | sort`.

  QA scenarios (MANDATORY - task incomplete without these):
  ```
  Scenario: full CLI and MCP compatibility
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && bun test tests/e2e.compat.test.ts > .omo/evidence/task-13-e2e.txt 2>&1
    Expected: Test exits 0 and covers CLI lifecycle plus MCP same-store operations.
    Evidence: .omo/evidence/task-13-e2e.txt

  Scenario: full suite and smoke
    Tool:     bash
    Steps:    cd /Users/iyen/projects/openclaw-ticket-service && { bun install --frozen-lockfile && bun run typecheck && bun run lint && bun test && (bun run qa:all || { bun run scripts/cli-smoke.ts && bun run scripts/mcp-smoke.ts; }); } > .omo/evidence/task-13-full-suite.txt 2>&1
    Expected: Command exits 0, with typecheck, lint, tests, CLI smoke, and MCP smoke all passing.
    Evidence: .omo/evidence/task-13-full-suite.txt
  ```

  Commit: YES | Message: `test(e2e): verify ticket service compatibility` | Files: [`tests/e2e.compat.test.ts`, `scripts/cli-smoke.ts`, `scripts/mcp-smoke.ts`, `.omo/evidence/`]

## Final verification wave (MANDATORY - after all implementation tasks)
> Runs in PARALLEL. ALL must APPROVE. Surface results to the caller and wait for an explicit "okay" before declaring complete.
- [ ] F1. Plan compliance audit - every task done, every acceptance criterion met
- [ ] F2. Code quality review - diagnostics clean, idioms match, no dead code
- [ ] F3. Real manual QA - every QA scenario executed with evidence captured
- [ ] F4. Scope fidelity - nothing extra shipped beyond Must-Have, nothing Must-NOT-Have introduced

## Commit strategy
- One logical change per commit. Conventional Commits (`<type>(<scope>): <subject>` body + footer).
- Atomic: every commit builds and passes tests on its own.
- No "WIP" / "fix typo squash later" commits on the final branch - clean up before merge.
- Reference the plan file path in the final commit footer: `Plan: .omo/plans/openclaw-ticket-service.md`.
- Because `/Users/iyen/projects/openclaw-ticket-service` is not currently a Git repository, Task 1 must initialize Git or confirm a newly initialized repo before any commit instructions are executed.
- Do not include generated `.omo/evidence/` in product commits unless the executor's workflow requires evidence artifacts in-repo; if omitted from Git, still capture and surface the evidence paths.

## Success criteria
- All Must-Have shipped; all QA scenarios pass with captured evidence; F1-F4 approved; commit history clean.
- `openclaw-ticket` CLI and MCP stdio server both operate against the same shared service/store layer.
- Existing `ticket.sh` JSON files remain readable; newly created JSON files preserve the existing field/default/index/archive contract.
- `OPENCLAW_TICKET_HOME` writes are isolated and verified; no automated test or QA writes to the real `~/.openclaw/tickets`.
- Strict TypeScript, Biome lint, Bun tests, real CLI QA, and real MCP stdio QA all pass from a clean checkout.

## EXPAND leads
- Golden shell diff harness: generate temp-store outputs from `ticket.sh` and compare the TypeScript service JSON command-by-command.
- Index repair command: later add a read-only audit and explicit repair flow for historical `index.json` drift.
- MCP resources/prompts: later expose ticket files as MCP resources once tool parity is stable.
- Distribution: later decide whether to publish as a private npm package, Bun standalone executable, or OpenClaw bundled tool.
