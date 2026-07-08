import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { z } from "zod"

type JsonRpcResponse = {
  readonly jsonrpc: "2.0"
  readonly id?: number
  readonly result?: unknown
  readonly error?: unknown
}

const McpTextResultSchema = z.object({
  content: z.array(z.object({ type: z.literal("text"), text: z.string() })),
})

function mcpText(result: unknown): string {
  const parsed = McpTextResultSchema.parse(result)
  return parsed.content.map((item) => item.text).join("\n")
}

async function runMcp(
  messages: readonly object[],
  storeRoot: string,
): Promise<readonly JsonRpcResponse[]> {
  const process = Bun.spawn(["bun", "run", "src/mcp.ts"], {
    cwd: import.meta.dir.replace(/\/tests$/, ""),
    env: { ...Bun.env, TICKET_FLOW_HOME: storeRoot },
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  })

  for (const message of messages) {
    process.stdin.write(`${JSON.stringify(message)}\n`)
  }
  process.stdin.end()

  const [stdout, exitCode] = await Promise.all([
    new Response(process.stdout).text(),
    process.exited,
  ])
  expect(exitCode).toBe(0)

  return stdout
    .split("\n")
    .filter((line) => line.trim().length > 0)
    .map((line) => JSON.parse(line) as JsonRpcResponse)
}

describe("MCP stdio server", () => {
  let storeRoot = ""

  beforeEach(async () => {
    storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-mcp-test-"))
    await mkdir(join(storeRoot, "active"), { recursive: true })
    await mkdir(join(storeRoot, "archive"), { recursive: true })
  })

  afterEach(async () => {
    await rm(storeRoot, { recursive: true, force: true })
  })

  test("Given a stdio MCP client When listing and calling tools Then ticket_create is available and works", async () => {
    const responses = await runMcp(
      [
        {
          jsonrpc: "2.0",
          id: 1,
          method: "initialize",
          params: {
            protocolVersion: "2025-06-18",
            capabilities: {},
            clientInfo: { name: "ticket-flow-test", version: "0.0.0" },
          },
        },
        { jsonrpc: "2.0", method: "notifications/initialized" },
        { jsonrpc: "2.0", id: 2, method: "tools/list" },
        {
          jsonrpc: "2.0",
          id: 3,
          method: "tools/call",
          params: {
            name: "ticket_create",
            arguments: {
              title: "Create through MCP",
              priority: "medium",
              type: "feature",
              source: "agent:test",
            },
          },
        },
      ],
      storeRoot,
    )

    const tools = responses.find((response) => response.id === 2)
    expect(JSON.stringify(tools?.result)).toContain("ticket_create")

    const created = responses.find((response) => response.id === 3)
    expect(JSON.stringify(created?.result)).toContain("T-")
    expect(JSON.stringify(created?.result)).toContain("Create through MCP")
  })

  test("Given a source ticket store When ticket_import is called Then it imports through the MCP surface", async () => {
    const sourceRoot = await mkdtemp(join(tmpdir(), "ticket-flow-mcp-import-source-"))
    await mkdir(join(sourceRoot, "active"), { recursive: true })
    try {
      await writeTicketFixture(join(sourceRoot, "active", "T-20260708-004.json"), {
        id: "T-20260708-004",
        title: "MCP import active",
        status: "open",
        updated: "2026-07-08T04:00:00Z",
      })

      const responses = await runMcp(
        [
          {
            jsonrpc: "2.0",
            id: 1,
            method: "initialize",
            params: {
              protocolVersion: "2025-06-18",
              capabilities: {},
              clientInfo: { name: "ticket-flow-test", version: "0.0.0" },
            },
          },
          { jsonrpc: "2.0", method: "notifications/initialized" },
          { jsonrpc: "2.0", id: 2, method: "tools/list" },
          {
            jsonrpc: "2.0",
            id: 3,
            method: "tools/call",
            params: {
              name: "ticket_import",
              arguments: { sourceRoot },
            },
          },
        ],
        storeRoot,
      )

      const tools = responses.find((response) => response.id === 2)
      expect(JSON.stringify(tools?.result)).toContain("ticket_import")

      const imported = responses.find((response) => response.id === 3)
      expect(JSON.parse(mcpText(imported?.result))).toEqual({ imported: 1, active: 1, archived: 0 })
      const ticket = JSON.parse(
        await readFile(join(storeRoot, "active", "T-20260708-004.json"), "utf8"),
      )
      expect(ticket.title).toBe("MCP import active")
    } finally {
      await rm(sourceRoot, { recursive: true, force: true })
    }
  })
})

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
