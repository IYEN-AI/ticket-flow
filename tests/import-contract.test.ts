import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { importTicketStore, listTickets } from "../src/store"
import { exists, writeTicketFixture } from "./support/store-fixtures"

describe("ticket import contract", () => {
  let storeRoot = ""

  beforeEach(async () => {
    storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-contract-"))
    await mkdir(join(storeRoot, "active"), { recursive: true })
    await mkdir(join(storeRoot, "archive", "2026-05"), { recursive: true })
    await writeFile(join(storeRoot, "index.json"), `{"version":1,"lastId":null,"tickets":{}}\n`)
  })

  afterEach(async () => {
    await rm(storeRoot, { recursive: true, force: true })
  })

  test("Given source active and archived tickets When importing Then active tickets are indexed and completed tickets are archived", async () => {
    const sourceRoot = await mkdtemp(join(tmpdir(), "ticket-flow-import-source-"))
    await mkdir(join(sourceRoot, "active"), { recursive: true })
    await mkdir(join(sourceRoot, "archive", "2026-06"), { recursive: true })
    try {
      await writeTicketFixture(join(sourceRoot, "active", "T-20260630-001.json"), {
        id: "T-20260630-001",
        title: "import active",
        status: "open",
        updated: "2026-06-30T10:00:00Z",
      })
      await writeTicketFixture(join(sourceRoot, "active", "T-20260630-002.json"), {
        id: "T-20260630-002",
        title: "import done from active",
        status: "done",
        updated: "2026-06-30T11:00:00Z",
        closed: "2026-07-01T00:00:00Z",
      })
      await writeTicketFixture(join(sourceRoot, "archive", "2026-06", "T-20260629-001.json"), {
        id: "T-20260629-001",
        title: "import archived",
        status: "done",
        updated: "2026-06-29T09:00:00Z",
        closed: null,
      })

      const summary = await importTicketStore(sourceRoot, {
        root: storeRoot,
        active: join(storeRoot, "active"),
        archive: join(storeRoot, "archive"),
        index: join(storeRoot, "index.json"),
      })

      const active = await listTickets({
        root: storeRoot,
        active: join(storeRoot, "active"),
        archive: join(storeRoot, "archive"),
        index: join(storeRoot, "index.json"),
      })
      const index = JSON.parse(await readFile(join(storeRoot, "index.json"), "utf8"))
      expect(summary).toEqual({ imported: 3, active: 1, archived: 2 })
      expect(active.map((ticket) => ticket.id)).toEqual(["T-20260630-001"])
      expect(index.tickets["T-20260630-001"]).toMatchObject({ title: "import active" })
      expect(index.tickets["T-20260630-002"]).toBeUndefined()
      expect(await exists(join(storeRoot, "archive", "2026-07", "T-20260630-002.json"))).toBe(true)
      expect(await exists(join(storeRoot, "archive", "2026-06", "T-20260629-001.json"))).toBe(true)
    } finally {
      await rm(sourceRoot, { recursive: true, force: true })
    }
  })

  test("Given a destination ticket with the same ID When importing Then it fails without overwriting", async () => {
    const sourceRoot = await mkdtemp(join(tmpdir(), "ticket-flow-import-source-"))
    await mkdir(join(sourceRoot, "active"), { recursive: true })
    try {
      await writeTicketFixture(join(storeRoot, "active", "T-20260630-003.json"), {
        id: "T-20260630-003",
        title: "existing destination",
        status: "open",
        updated: "2026-06-30T10:00:00Z",
      })
      await writeTicketFixture(join(sourceRoot, "active", "T-20260630-003.json"), {
        id: "T-20260630-003",
        title: "source duplicate",
        status: "open",
        updated: "2026-06-30T11:00:00Z",
      })

      await expect(
        importTicketStore(sourceRoot, {
          root: storeRoot,
          active: join(storeRoot, "active"),
          archive: join(storeRoot, "archive"),
          index: join(storeRoot, "index.json"),
        }),
      ).rejects.toThrow("duplicate destination ticket T-20260630-003")

      const destination = JSON.parse(
        await readFile(join(storeRoot, "active", "T-20260630-003.json"), "utf8"),
      )
      expect(destination.title).toBe("existing destination")
    } finally {
      await rm(sourceRoot, { recursive: true, force: true })
    }
  })
})
