import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { getTicket, listTickets, updateStatus } from "../src/store"
import { resolveStorePaths } from "../src/store/paths"

describe("historical ticket contract", () => {
  let storeRoot = ""

  beforeEach(async () => {
    storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-contract-"))
    await mkdir(join(storeRoot, "active"), { recursive: true })
    await mkdir(join(storeRoot, "archive", "2026-05"), { recursive: true })
    await writeFile(join(storeRoot, "index.json"), `{"version":1,"lastId":null,"tickets":{}}\n`)
  })

  afterEach(async () => {
    delete Bun.env["TICKET_FLOW_HOME"]
    delete Bun.env["OPENCLAW_TICKET_HOME"]
    await rm(storeRoot, { recursive: true, force: true })
  })

  test("Given historical active tickets with missing tags and string artifacts When listing Then they remain readable", async () => {
    await writeFile(
      join(storeRoot, "active", "T-20260623-001.json"),
      JSON.stringify({
        id: "T-20260623-001",
        title: "legacy ticket",
        status: "open",
        priority: "2",
        type: "triage",
        source: { type: "discord", ref: "1", sender: { name: "legacy" } },
        artifacts: ["legacy artifact"],
        links: {},
        created: "2026-06-23T00:00:00Z",
        updated: "2026-06-23T00:00:00Z",
      }),
    )

    const tickets = await listTickets({
      root: storeRoot,
      active: join(storeRoot, "active"),
      archive: join(storeRoot, "archive"),
      index: join(storeRoot, "index.json"),
    })

    expect(tickets).toHaveLength(1)
    expect(tickets[0]?.tags).toEqual([])
    expect(tickets[0]?.priority).toBe("2")
    expect(tickets[0]?.artifacts[0]).toMatchObject({ type: "artifact", value: "legacy artifact" })
  })

  test("Given an active ticket When status reaches done Then it archives the file and removes the index entry", async () => {
    await writeFile(
      join(storeRoot, "active", "T-20260701-001.json"),
      JSON.stringify({
        id: "T-20260701-001",
        title: "archive me",
        status: "review",
        priority: "medium",
        type: "chore",
        source: null,
        goal: "",
        acceptance: [],
        tags: [],
        artifacts: [{ type: "artifact", value: "proof", ts: "2026-07-01T00:00:00Z" }],
        links: { github_issues: [], prs: [], threads: [], cron_jobs: [] },
        parent: null,
        children: [],
        assignee: "iyen",
        created: "2026-07-01T00:00:00Z",
        updated: "2026-07-01T00:00:00Z",
        closed: null,
        log: [],
      }),
    )
    await writeFile(
      join(storeRoot, "index.json"),
      JSON.stringify({
        version: 1,
        lastId: "T-20260701-001",
        tickets: {
          "T-20260701-001": {
            title: "archive me",
            status: "review",
            priority: "medium",
            type: "chore",
            updated: "2026-07-01T00:00:00Z",
          },
        },
      }),
    )

    await updateStatus(
      { id: "T-20260701-001", status: "done" },
      {
        root: storeRoot,
        active: join(storeRoot, "active"),
        archive: join(storeRoot, "archive"),
        index: join(storeRoot, "index.json"),
      },
    )

    const archived = await getTicket("T-20260701-001", {
      root: storeRoot,
      active: join(storeRoot, "active"),
      archive: join(storeRoot, "archive"),
      index: join(storeRoot, "index.json"),
    })
    const index = JSON.parse(await readFile(join(storeRoot, "index.json"), "utf8"))
    const activeExists = await exists(join(storeRoot, "active", "T-20260701-001.json"))
    const archiveExists = await exists(join(storeRoot, "archive", "2026-07", "T-20260701-001.json"))
    expect(archived.status).toBe("done")
    expect(archived.closed).not.toBeNull()
    expect(activeExists).toBe(false)
    expect(archiveExists).toBe(true)
    expect(index.tickets["T-20260701-001"]).toBeUndefined()
  })

  test("Given ticket-flow home env When resolving store paths Then it uses the native override", () => {
    Bun.env["TICKET_FLOW_HOME"] = "/tmp/ticket-flow-home"
    Bun.env["OPENCLAW_TICKET_HOME"] = "/tmp/openclaw-home"

    const paths = resolveStorePaths()

    expect(paths.root).toBe("/tmp/ticket-flow-home")
    expect(paths.active).toBe("/tmp/ticket-flow-home/active")
  })

  test("Given only the old openclaw home env When resolving store paths Then it is ignored", () => {
    delete Bun.env["TICKET_FLOW_HOME"]
    Bun.env["OPENCLAW_TICKET_HOME"] = "/tmp/openclaw-home"

    const paths = resolveStorePaths()

    expect(paths.root).not.toBe("/tmp/openclaw-home")
    expect(paths.active).not.toBe("/tmp/openclaw-home/active")
  })

  test("Given no ticket-flow home env When resolving store paths Then it uses the native default", () => {
    delete Bun.env["TICKET_FLOW_HOME"]
    delete Bun.env["OPENCLAW_TICKET_HOME"]

    const paths = resolveStorePaths()

    expect(paths.root).toEndWith("/.ticket-flow/tickets")
    expect(paths.active).toEndWith("/.ticket-flow/tickets/active")
  })
})

async function exists(path: string): Promise<boolean> {
  try {
    await stat(path)
    return true
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false
    }
    throw error
  }
}
