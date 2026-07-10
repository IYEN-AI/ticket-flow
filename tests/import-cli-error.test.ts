import { expect, test } from "bun:test"
import { mkdir, mkdtemp, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

test("Given a missing source root When import runs Then it exits with an error", async () => {
  const storeRoot = await mkdtemp(join(tmpdir(), "ticket-flow-import-cli-error-"))
  await mkdir(join(storeRoot, "active"), { recursive: true })
  await mkdir(join(storeRoot, "archive"), { recursive: true })
  const sourceRoot = join(storeRoot, "missing-import-source")
  try {
    const process = Bun.spawn(["bun", "run", "src/cli.ts", "import", sourceRoot], {
      cwd: import.meta.dir.replace(/\/tests$/, ""),
      env: { ...Bun.env, TICKET_FLOW_HOME: storeRoot },
      stdout: "pipe",
      stderr: "pipe",
    })
    const [stderr, exitCode] = await Promise.all([
      new Response(process.stderr).text(),
      process.exited,
    ])

    expect(exitCode).toBe(1)
    expect(stderr).toContain(`invalid import source ${sourceRoot}`)
  } finally {
    await rm(storeRoot, { recursive: true, force: true })
  }
})
