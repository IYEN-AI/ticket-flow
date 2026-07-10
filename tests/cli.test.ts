import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { access, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

type CliRun = {
  readonly stdout: string
  readonly stderr: string
  readonly exitCode: number
}

async function runCli(
  args: readonly string[],
  storeRoot: string,
  env: Record<string, string> = {},
): Promise<CliRun> {
  const process = Bun.spawn(["bun", "run", "src/cli.ts", ...args], {
    cwd: import.meta.dir.replace(/\/tests$/, ""),
    env: { ...Bun.env, ...env, TICKET_FLOW_HOME: storeRoot },
    stdout: "pipe",
    stderr: "pipe",
  })
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
    process.exited,
  ])
  return { stdout, stderr, exitCode }
}

describe("CLI compatibility", () => {
  let storeRoot = ""

  beforeEach(async () => {
    storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-test-"))
    await mkdir(join(storeRoot, "active"), { recursive: true })
    await mkdir(join(storeRoot, "archive"), { recursive: true })
  })

  afterEach(async () => {
    await rm(storeRoot, { recursive: true, force: true })
  })

  test("Given an empty ticket store When create runs Then it writes the existing JSON and index contract", async () => {
    const result = await runCli(
      [
        "create",
        "--title",
        "Preserve ticket contract",
        "--type",
        "feature",
        "--priority",
        "high",
        "--goal",
        "Move CLI into service repo",
        "--acceptance",
        "existing JSON is readable",
        "--tag",
        "mcp,cli",
        "--source",
        "discord:12345",
      ],
      storeRoot,
    )

    expect(result.exitCode).toBe(0)
    expect(result.stderr).toBe("")
    const id = result.stdout.trim()
    expect(id).toMatch(/^T-\d{8}-001$/)

    const ticket = JSON.parse(await readFile(join(storeRoot, "active", `${id}.json`), "utf8"))
    expect(ticket).toMatchObject({
      id,
      title: "Preserve ticket contract",
      status: "open",
      priority: "high",
      type: "feature",
      source: { type: "discord", ref: "12345" },
      goal: "Move CLI into service repo",
      acceptance: ["existing JSON is readable"],
      tags: ["mcp", "cli"],
      links: { github_issues: [], prs: [], threads: [], cron_jobs: [] },
      assignee: "iyen",
      closed: null,
    })

    const index = JSON.parse(await readFile(join(storeRoot, "index.json"), "utf8"))
    expect(index.tickets[id]).toMatchObject({
      title: "Preserve ticket contract",
      status: "open",
      priority: "high",
      type: "feature",
    })
  })

  test("Given a review transition without artifact When status runs Then it preserves the existing guardrail", async () => {
    const create = await runCli(["create", "--title", "Guard review"], storeRoot)
    const id = create.stdout.trim()

    await runCli(["status", id, "doing"], storeRoot)
    const result = await runCli(["status", id, "review"], storeRoot)

    expect(result.exitCode).toBe(1)
    expect(result.stderr).toContain("review requires --artifact")
  })

  test("Given no checkpoint fields When checkpoint runs Then it preserves the existing guardrail", async () => {
    const create = await runCli(["create", "--title", "Guard checkpoint"], storeRoot)
    const id = create.stdout.trim()

    const result = await runCli(["checkpoint", id], storeRoot)

    expect(result.exitCode).toBe(1)
    expect(result.stderr).toContain("checkpoint requires at least one field")
  })

  test("Given a source ticket store When import runs Then destination active tickets and index are populated", async () => {
    const sourceRoot = await mkdtemp(join(tmpdir(), "ticket-flow-cli-import-source-"))
    await mkdir(join(sourceRoot, "active"), { recursive: true })
    await mkdir(join(sourceRoot, "archive", "2026-06"), { recursive: true })
    try {
      await writeTicketFixture(join(sourceRoot, "active", "T-20260708-001.json"), {
        id: "T-20260708-001",
        title: "CLI import active",
        status: "open",
        updated: "2026-07-08T01:00:00Z",
      })
      await writeTicketFixture(join(sourceRoot, "active", "T-20260708-002.json"), {
        id: "T-20260708-002",
        title: "CLI import done",
        status: "done",
        updated: "2026-07-08T02:00:00Z",
        closed: "2026-07-08T02:30:00Z",
      })
      await writeTicketFixture(join(sourceRoot, "archive", "2026-06", "T-20260630-001.json"), {
        id: "T-20260630-001",
        title: "CLI import archived",
        status: "done",
        updated: "2026-06-30T02:00:00Z",
      })

      const result = await runCli(["import", sourceRoot], storeRoot)

      expect(result.exitCode).toBe(0)
      expect(result.stderr).toBe("")
      expect(result.stdout).toContain("imported 3 ticket(s): active=1 archived=2")
      const active = JSON.parse(
        await readFile(join(storeRoot, "active", "T-20260708-001.json"), "utf8"),
      )
      const index = JSON.parse(await readFile(join(storeRoot, "index.json"), "utf8"))
      expect(active.title).toBe("CLI import active")
      expect(Object.keys(index.tickets)).toEqual(["T-20260708-001"])
      expect(await exists(join(storeRoot, "archive", "2026-07", "T-20260708-002.json"))).toBe(true)
      expect(await exists(join(storeRoot, "archive", "2026-06", "T-20260630-001.json"))).toBe(true)
      expect(await exists(join(sourceRoot, "active", "T-20260708-001.json"))).toBe(true)
    } finally {
      await rm(sourceRoot, { recursive: true, force: true })
    }
  })

  test("Given duplicate destination ticket When import runs Then it fails without overwriting", async () => {
    const sourceRoot = await mkdtemp(join(tmpdir(), "ticket-flow-cli-import-source-"))
    await mkdir(join(sourceRoot, "active"), { recursive: true })
    try {
      await writeTicketFixture(join(storeRoot, "active", "T-20260708-003.json"), {
        id: "T-20260708-003",
        title: "destination original",
        status: "open",
        updated: "2026-07-08T01:00:00Z",
      })
      await writeTicketFixture(join(sourceRoot, "active", "T-20260708-003.json"), {
        id: "T-20260708-003",
        title: "source duplicate",
        status: "open",
        updated: "2026-07-08T02:00:00Z",
      })

      const result = await runCli(["import", sourceRoot], storeRoot)

      expect(result.exitCode).toBe(1)
      expect(result.stderr).toContain("duplicate destination ticket T-20260708-003")
      const destination = JSON.parse(
        await readFile(join(storeRoot, "active", "T-20260708-003.json"), "utf8"),
      )
      expect(destination.title).toBe("destination original")
    } finally {
      await rm(sourceRoot, { recursive: true, force: true })
    }
  })

  test("Given an existing ticket When clawhip event prints ticket.created Then it returns routeable JSON", async () => {
    const create = await runCli(
      ["create", "--title", "Print clawhip event", "--assignee", "codex"],
      storeRoot,
    )
    const id = create.stdout.trim()

    const result = await runCli(
      ["clawhip", "event", id, "--kind", "ticket.created", "--print"],
      storeRoot,
      { TICKET_FLOW_REPO_PATH: "/repo/ticket-flow" },
    )

    expect(result.exitCode).toBe(0)
    expect(result.stderr).toBe("")
    expect(JSON.parse(result.stdout)).toMatchObject({
      type: "ticket.created",
      payload: {
        provider: "ticket-flow",
        ticket_id: id,
        repo_path: "/repo/ticket-flow",
      },
    })
  })

  test("Given an unsupported clawhip event kind When printing Then the CLI rejects it", async () => {
    const create = await runCli(["create", "--title", "Reject clawhip event"], storeRoot)
    const id = create.stdout.trim()

    const result = await runCli(
      ["clawhip", "event", id, "--kind", "ticket.done", "--print"],
      storeRoot,
    )

    expect(result.exitCode).toBe(1)
    expect(result.stderr).toContain("Invalid option")
  })
})

async function exists(path: string): Promise<boolean> {
  try {
    await access(path)
    return true
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false
    }
    throw error
  }
}

async function writeTicketFixture(
  path: string,
  overrides: {
    readonly id: string
    readonly title: string
    readonly status: "open" | "doing" | "review" | "blocked" | "done"
    readonly updated: string
    readonly closed?: string | null
  },
): Promise<void> {
  await writeFile(
    path,
    `${JSON.stringify({
      id: overrides.id,
      title: overrides.title,
      status: overrides.status,
      priority: "medium",
      type: "chore",
      source: null,
      goal: "",
      acceptance: [],
      tags: [],
      artifacts: [],
      links: { github_issues: [], prs: [], threads: [], cron_jobs: [] },
      parent: null,
      children: [],
      assignee: "iyen",
      created: "2026-07-08T00:00:00Z",
      updated: overrides.updated,
      closed: overrides.closed ?? null,
      log: [],
    })}\n`,
  )
}
