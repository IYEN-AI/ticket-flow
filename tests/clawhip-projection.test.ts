import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { buildClawhipEvent, sendClawhipEvent } from "../src/clawhip"
import { createTicket } from "../src/store/create"
import { resolveStorePaths } from "../src/store/paths"

describe("clawhip event projection", () => {
  let storeRoot = ""

  beforeEach(async () => {
    storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-clawhip-test-"))
    await mkdir(join(storeRoot, "active"), { recursive: true })
    await mkdir(join(storeRoot, "archive"), { recursive: true })
  })

  afterEach(async () => {
    await rm(storeRoot, { recursive: true, force: true })
  })

  test("Given a ticket with private logs and artifacts When projecting ticket.created Then payload is compact and routeable", async () => {
    const paths = resolveStorePaths(storeRoot)
    const ticket = await createTicket(
      {
        title: "Route ticket to clawhip",
        priority: "high",
        type: "feature",
        goal: "private goal must not leak",
        acceptance: ["private acceptance must not leak"],
        tags: ["clawhip"],
        source: "manual:qa",
        assignee: "codex",
      },
      paths,
    )
    const withPrivateFields = {
      ...ticket,
      artifacts: [{ type: "artifact", value: "SECRET_ARTIFACT", ts: ticket.updated }],
      log: [...ticket.log, { ts: ticket.updated, action: "note", note: "SECRET_LOG" }],
    }

    const event = buildClawhipEvent({
      kind: "ticket.created",
      ticket: withPrivateFields,
      repoPath: "/repo/ticket-flow",
    })

    expect(event).toEqual({
      type: "ticket.created",
      payload: {
        provider: "ticket-flow",
        event: "ticket.created",
        ticket_id: ticket.id,
        title: "Route ticket to clawhip",
        status: "open",
        priority: "high",
        type: "feature",
        assignee: "codex",
        summary: `${ticket.id} [open] Route ticket to clawhip`,
        repo_path: "/repo/ticket-flow",
        worktree_path: "/repo/ticket-flow",
        repo_name: "ticket-flow",
        event_timestamp: ticket.updated,
      },
    })
    expect(JSON.stringify(event)).not.toContain("SECRET_ARTIFACT")
    expect(JSON.stringify(event)).not.toContain("SECRET_LOG")
  })

  test("Given an unsupported ticket event kind When projecting Then parsing rejects the event", async () => {
    const paths = resolveStorePaths(storeRoot)
    const ticket = await createTicket(
      {
        title: "Reject bad event",
        type: "chore",
        priority: "medium",
        goal: "",
        assignee: "iyen",
        acceptance: [],
        tags: [],
      },
      paths,
    )

    expect(() =>
      buildClawhipEvent({
        kind: "ticket.done",
        ticket,
        repoPath: "/repo/ticket-flow",
      }),
    ).toThrow()
  })
})

describe("clawhip delivery", () => {
  test("Given a clawhip daemon URL When sending an event Then it posts the IncomingEvent JSON", async () => {
    const received: string[] = []
    const server = Bun.serve({
      port: 0,
      fetch: async (request) => {
        received.push(await request.text())
        return Response.json(
          { ok: true, type: "ticket.created", event_id: "evt-test" },
          { status: 202 },
        )
      },
    })

    try {
      const result = await sendClawhipEvent({
        url: `http://127.0.0.1:${server.port}`,
        event: {
          type: "ticket.created",
          payload: {
            provider: "ticket-flow",
            event: "ticket.created",
            ticket_id: "T-20260704-001",
            title: "Send to clawhip",
            status: "open",
            summary: "T-20260704-001 [open] Send to clawhip",
            event_timestamp: "2026-07-04T00:00:00Z",
          },
        },
      })

      expect(result).toEqual({ ok: true, status: 202 })
      expect(JSON.parse(received[0] ?? "")).toMatchObject({
        type: "ticket.created",
        payload: {
          provider: "ticket-flow",
          ticket_id: "T-20260704-001",
        },
      })
    } finally {
      await server.stop()
    }
  })

  test("Given clawhip is unavailable When sending best-effort Then failure is returned instead of thrown", async () => {
    const result = await sendClawhipEvent({
      url: "http://127.0.0.1:9",
      event: {
        type: "ticket.created",
        payload: {
          provider: "ticket-flow",
          event: "ticket.created",
          ticket_id: "T-20260704-001",
          title: "Downstream down",
          status: "open",
          summary: "T-20260704-001 [open] Downstream down",
          event_timestamp: "2026-07-04T00:00:00Z",
        },
      },
      mode: "best-effort",
      timeoutMs: 50,
    })

    expect(result.ok).toBe(false)
  })
})
