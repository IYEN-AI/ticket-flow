import { readdir } from "node:fs/promises"
import { join } from "node:path"
import { idsInDirectory, readIndex } from "./io"
import type { TicketStorePaths } from "./paths"
import { utcToday } from "./time"

export async function nextTicketId(paths: TicketStorePaths): Promise<string> {
  const today = utcToday()
  const ids = new Set<string>()
  const index = await readIndex(paths)
  for (const id of Object.keys(index.tickets)) {
    ids.add(id)
  }
  for (const id of await idsInDirectory(paths.active)) {
    ids.add(id)
  }
  const archiveMonths = await readdir(paths.archive, { withFileTypes: true })
  for (const month of archiveMonths.filter((entry) => entry.isDirectory())) {
    for (const id of await idsInDirectory(join(paths.archive, month.name))) {
      ids.add(id)
    }
  }
  const next =
    Array.from(ids)
      .filter((id) => id.startsWith(`T-${today}-`))
      .map((id) => Number(id.slice(-3)))
      .filter((value) => Number.isInteger(value))
      .reduce((max, value) => Math.max(max, value), 0) + 1
  return `T-${today}-${next.toString().padStart(3, "0")}`
}
