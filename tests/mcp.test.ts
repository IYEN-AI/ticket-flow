import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

type JsonRpcResponse = {
  readonly jsonrpc: "2.0"
  readonly id?: number
  readonly result?: unknown
  readonly error?: unknown
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
})
