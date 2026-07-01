import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

type CliRun = {
  readonly stdout: string
  readonly stderr: string
  readonly exitCode: number
}

async function runCli(args: readonly string[], storeRoot: string): Promise<CliRun> {
  const process = Bun.spawn(["bun", "run", "src/cli.ts", ...args], {
    cwd: import.meta.dir.replace(/\/tests$/, ""),
    env: { ...Bun.env, OPENCLAW_TICKET_HOME: storeRoot },
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

  test("Given an empty OpenClaw ticket store When create runs Then it writes the existing JSON and index contract", async () => {
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
})
