import { mkdir, readFile } from "node:fs/promises"
import { homedir } from "node:os"
import { join } from "node:path"
import { writeJson } from "./io"

export type TicketStorePaths = {
  readonly root: string
  readonly active: string
  readonly archive: string
  readonly index: string
}

export function resolveStorePaths(root = Bun.env["OPENCLAW_TICKET_HOME"]): TicketStorePaths {
  const resolvedRoot = root ?? join(homedir(), ".openclaw", "tickets")
  return {
    root: resolvedRoot,
    active: join(resolvedRoot, "active"),
    archive: join(resolvedRoot, "archive"),
    index: join(resolvedRoot, "index.json"),
  }
}

export async function ensureStore(paths: TicketStorePaths): Promise<void> {
  await mkdir(paths.active, { recursive: true })
  await mkdir(paths.archive, { recursive: true })
  try {
    await readFile(paths.index, "utf8")
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      await writeJson(paths.index, { version: 1, lastId: null, tickets: {} })
      return
    }
    throw error
  }
}
