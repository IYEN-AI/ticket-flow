import { readdir, readFile, rename, writeFile } from "node:fs/promises"
import { basename, join } from "node:path"
import { TicketNotFoundError } from "../errors"
import { IndexSchema, type Ticket, type TicketIndex, TicketSchema } from "../schema"
import type { TicketStorePaths } from "./paths"

export async function readActiveTicket(paths: TicketStorePaths, id: string): Promise<Ticket> {
  try {
    return await readTicketFile(join(paths.active, `${id}.json`))
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      throw new TicketNotFoundError(id)
    }
    throw error
  }
}

export async function readTicketFile(file: string): Promise<Ticket> {
  return TicketSchema.parse(JSON.parse(await readFile(file, "utf8")))
}

export async function readIndex(paths: TicketStorePaths): Promise<TicketIndex> {
  try {
    return IndexSchema.parse(JSON.parse(await readFile(paths.index, "utf8")))
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return { version: 1, tickets: {} }
    }
    throw error
  }
}

export async function upsertIndex(paths: TicketStorePaths, ticket: Ticket): Promise<void> {
  const index = await readIndex(paths)
  await writeJson(paths.index, {
    ...index,
    lastId: ticket.id,
    tickets: {
      ...index.tickets,
      [ticket.id]: {
        title: ticket.title,
        status: ticket.status,
        priority: ticket.priority,
        type: ticket.type,
        updated: ticket.updated,
      },
    },
  })
}

export async function removeIndex(paths: TicketStorePaths, id: string): Promise<void> {
  const index = await readIndex(paths)
  const nextTickets = { ...index.tickets }
  delete nextTickets[id]
  await writeJson(paths.index, { ...index, tickets: nextTickets })
}

export async function writeTicket(paths: TicketStorePaths, ticket: Ticket): Promise<void> {
  await writeJson(join(paths.active, `${ticket.id}.json`), ticket)
}

export async function writeJson(file: string, value: unknown): Promise<void> {
  const target = `${file}.tmp-${crypto.randomUUID()}`
  await writeFile(target, `${JSON.stringify(value, null, 2)}\n`, "utf8")
  await rename(target, file)
}

export async function idsInDirectory(directory: string): Promise<readonly string[]> {
  try {
    const entries = await readdir(directory, { withFileTypes: true })
    return entries
      .filter((entry) => entry.isFile() && /^T-\d{8}-\d{3}\.json$/.test(entry.name))
      .map((entry) => basename(entry.name, ".json"))
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return []
    }
    throw error
  }
}
