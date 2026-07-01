import { readdir } from "node:fs/promises"
import { basename, join } from "node:path"
import { TicketNotFoundError } from "../errors"
import type { Ticket, TicketStatus } from "../schema"
import { readActiveTicket, readTicketFile } from "./io"
import type { TicketStorePaths } from "./paths"
import { ensureStore, resolveStorePaths } from "./paths"
import { ageMinutesSince } from "./time"

export async function listTickets(
  paths = resolveStorePaths(),
  status?: TicketStatus,
): Promise<readonly Ticket[]> {
  await ensureStore(paths)
  const entries = await readdir(paths.active, { withFileTypes: true })
  const tickets = await Promise.all(
    entries
      .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
      .map((entry) => readActiveTicket(paths, basename(entry.name, ".json"))),
  )
  return tickets
    .filter((ticket) => status === undefined || ticket.status === status)
    .sort((left, right) => left.id.localeCompare(right.id))
}

export type AgentActionTicket = {
  readonly id: string
  readonly status: Ticket["status"]
  readonly owner: string
  readonly command: string
  readonly ageMinutes?: number
}

export async function listAgentActions(
  paths = resolveStorePaths(),
  staleMinutes?: number,
): Promise<readonly AgentActionTicket[]> {
  const tickets = await listTickets(paths)
  const now = Date.now()
  return tickets.flatMap((ticket) => {
    const nextAction = ticket.current?.next_action
    if (nextAction?.type !== "agent_action") {
      return []
    }
    const ageMinutes = ageMinutesSince(ticket.updated, now)
    if (staleMinutes !== undefined && (ageMinutes === undefined || ageMinutes < staleMinutes)) {
      return []
    }
    return [
      {
        id: ticket.id,
        status: ticket.status,
        owner: nextAction.owner ?? ticket.assignee,
        command: nextAction.command ?? ticket.current?.next ?? "",
        ...(ageMinutes === undefined ? {} : { ageMinutes }),
      },
    ]
  })
}

export async function getTicket(id: string, paths = resolveStorePaths()): Promise<Ticket> {
  await ensureStore(paths)
  try {
    return await readActiveTicket(paths, id)
  } catch (error) {
    if (!(error instanceof TicketNotFoundError)) {
      throw error
    }
  }
  return readArchivedTicket(paths, id)
}

async function readArchivedTicket(paths: TicketStorePaths, id: string): Promise<Ticket> {
  const monthDirs = await readdir(paths.archive, { withFileTypes: true })
  for (const monthDir of monthDirs.filter((entry) => entry.isDirectory())) {
    const file = join(paths.archive, monthDir.name, `${id}.json`)
    try {
      return await readTicketFile(file)
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT") {
        continue
      }
      throw error
    }
  }
  throw new TicketNotFoundError(id)
}
