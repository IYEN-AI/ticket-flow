import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { createTicket } from "../src/store/create"
import { checkpointTicket } from "../src/store/mutations"
import { resolveStorePaths } from "../src/store/paths"
import { updateStatus } from "../src/store/status"

describe("clawhip auto emit", () => {
  let storeRoot = ""

  beforeEach(async () => {
    storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-clawhip-auto-test-"))
    await mkdir(join(storeRoot, "active"), { recursive: true })
    await mkdir(join(storeRoot, "archive"), { recursive: true })
  })

  afterEach(async () => {
    delete Bun.env["TICKET_FLOW_CLAWHIP"]
    delete Bun.env["TICKET_FLOW_CLAWHIP_URL"]
    delete Bun.env["TICKET_FLOW_CLAWHIP_TIMEOUT_MS"]
    delete Bun.env["TICKET_FLOW_REPO_PATH"]
    await rm(storeRoot, { recursive: true, force: true })
  })

  test("Given clawhip auto emit is enabled When creating a ticket Then it posts ticket.created after storing the ticket", async () => {
    const received: string[] = []
    const server = serveClawhipSink(received)

    try {
      enableClawhip(boundPort(server))
      const paths = resolveStorePaths(storeRoot)

      const ticket = await createTicket(newTicket("Auto emit to clawhip"), paths)

      const stored = JSON.parse(
        await readFile(join(storeRoot, "active", `${ticket.id}.json`), "utf8"),
      )
      expect(stored.id).toBe(ticket.id)
      expect(JSON.parse(received[0] ?? "")).toMatchObject({
        type: "ticket.created",
        payload: {
          provider: "ticket-flow",
          ticket_id: ticket.id,
          repo_path: "/repo/ticket-flow",
        },
      })
    } finally {
      await server.stop()
    }
  })

  test("Given clawhip auto emit is enabled but daemon is down When creating a ticket Then the ticket mutation still succeeds", async () => {
    Bun.env["TICKET_FLOW_CLAWHIP"] = "1"
    Bun.env["TICKET_FLOW_CLAWHIP_URL"] = "http://127.0.0.1:9"
    Bun.env["TICKET_FLOW_CLAWHIP_TIMEOUT_MS"] = "50"
    Bun.env["TICKET_FLOW_REPO_PATH"] = "/repo/ticket-flow"
    const paths = resolveStorePaths(storeRoot)

    const ticket = await createTicket(newTicket("Keep ticket when clawhip is down"), paths)

    const stored = JSON.parse(
      await readFile(join(storeRoot, "active", `${ticket.id}.json`), "utf8"),
    )
    const index = JSON.parse(await readFile(join(storeRoot, "index.json"), "utf8"))
    expect(stored.title).toBe("Keep ticket when clawhip is down")
    expect(index.tickets[ticket.id].title).toBe("Keep ticket when clawhip is down")
  })

  test("Given clawhip auto emit is enabled When status changes Then ticket.status_changed is posted", async () => {
    const received: string[] = []
    const server = serveClawhipSink(received)

    try {
      enableClawhip(boundPort(server))
      const paths = resolveStorePaths(storeRoot)
      const ticket = await createTicket(newTicket("Emit status change"), paths)
      received.length = 0

      await updateStatus({ id: ticket.id, status: "doing" }, paths)

      expect(JSON.parse(received[0] ?? "")).toMatchObject({
        type: "ticket.status_changed",
        payload: {
          provider: "ticket-flow",
          ticket_id: ticket.id,
          from_status: "open",
          to_status: "doing",
        },
      })
    } finally {
      await server.stop()
    }
  })

  test("Given clawhip auto emit is enabled When checkpoint sets agent action and blocker Then checkpoint events are posted", async () => {
    const received: string[] = []
    const server = serveClawhipSink(received)

    try {
      enableClawhip(boundPort(server))
      const paths = resolveStorePaths(storeRoot)
      const ticket = await createTicket(newTicket("Emit checkpoint"), paths)
      received.length = 0

      await checkpointTicket(
        {
          id: ticket.id,
          phase: "qa",
          blocker: "waiting on qa",
          nextType: "agent_action",
          nextCommand: "bun test",
          nextOwner: "codex",
        },
        paths,
      )

      const eventTypes = received.map((body) => JSON.parse(body).type)
      expect(eventTypes).toEqual([
        "ticket.checkpointed",
        "ticket.agent_action_available",
        "ticket.blocked",
      ])
      expect(JSON.parse(received[1] ?? "")).toMatchObject({
        payload: {
          next_action_type: "agent_action",
          next_action_command: "bun test",
          next_action_owner: "codex",
        },
      })
    } finally {
      await server.stop()
    }
  })
})

function serveClawhipSink(received: string[]): ReturnType<typeof Bun.serve> {
  return Bun.serve({
    port: 0,
    fetch: async (request) => {
      received.push(await request.text())
      return Response.json(
        { ok: true, type: "ticket.event", event_id: "evt-test" },
        { status: 202 },
      )
    },
  })
}

function enableClawhip(port: number): void {
  Bun.env["TICKET_FLOW_CLAWHIP"] = "1"
  Bun.env["TICKET_FLOW_CLAWHIP_URL"] = `http://127.0.0.1:${port}`
  Bun.env["TICKET_FLOW_REPO_PATH"] = "/repo/ticket-flow"
}

function boundPort(server: ReturnType<typeof Bun.serve>): number {
  if (server.port === undefined) {
    throw new Error("test server did not bind a port")
  }
  return server.port
}

function newTicket(title: string) {
  return {
    title,
    type: "chore",
    priority: "medium",
    goal: "",
    assignee: "iyen",
    acceptance: [],
    tags: [],
  }
}
