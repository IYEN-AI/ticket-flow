import { mkdir, readdir, stat } from "node:fs/promises"
import { dirname, join } from "node:path"
import {
  DuplicateDestinationTicketError,
  DuplicateSourceTicketError,
  InvalidImportSourceError,
} from "../errors"
import type { Ticket } from "../schema"
import {
  idsInDirectory,
  readTicketFile,
  removeIndex,
  upsertIndex,
  writeJson,
  writeTicket,
} from "./io"
import type { TicketStorePaths } from "./paths"
import { ensureStore } from "./paths"

export type ImportTicketStoreSummary = {
  readonly imported: number
  readonly active: number
  readonly archived: number
}

type SourceTicket = {
  readonly ticket: Ticket
  readonly sourceArchiveMonth?: string
}

export async function importTicketStore(
  sourceRoot: string,
  destination: TicketStorePaths,
): Promise<ImportTicketStoreSummary> {
  const sourceTickets = await readImportSourceTickets(sourceRoot)
  rejectDuplicateSources(sourceTickets)
  await ensureStore(destination)
  await rejectDestinationCollisions(destination, sourceTickets)

  let active = 0
  let archived = 0
  for (const sourceTicket of sourceTickets) {
    if (sourceTicket.ticket.status === "done") {
      await writeArchivedTicket(destination, sourceTicket)
      await removeIndex(destination, sourceTicket.ticket.id)
      archived += 1
    } else {
      await writeTicket(destination, sourceTicket.ticket)
      await upsertIndex(destination, sourceTicket.ticket)
      active += 1
    }
  }
  return { imported: sourceTickets.length, active, archived }
}

async function readImportSourceTickets(sourceRoot: string): Promise<readonly SourceTicket[]> {
  try {
    const sourceMetadata = await stat(sourceRoot)
    if (!sourceMetadata.isDirectory()) {
      throw new InvalidImportSourceError(sourceRoot)
    }
    return await readSourceTickets(sourceRoot)
  } catch (error) {
    if (error instanceof InvalidImportSourceError) {
      throw error
    }
    if (error instanceof Error && "code" in error) {
      throw new InvalidImportSourceError(sourceRoot, { cause: error })
    }
    throw error
  }
}

async function readSourceTickets(sourceRoot: string): Promise<readonly SourceTicket[]> {
  return [
    ...(await readActiveSourceTickets(sourceRoot)),
    ...(await readArchivedSourceTickets(sourceRoot)),
  ]
}

async function readActiveSourceTickets(sourceRoot: string): Promise<readonly SourceTicket[]> {
  const activeRoot = join(sourceRoot, "active")
  const ids = await idsInDirectory(activeRoot)
  return Promise.all(
    ids.map(async (id) => ({
      ticket: await readTicketFile(join(activeRoot, `${id}.json`)),
    })),
  )
}

async function readArchivedSourceTickets(sourceRoot: string): Promise<readonly SourceTicket[]> {
  const archiveRoot = join(sourceRoot, "archive")
  const monthDirs = await archiveMonthDirs(archiveRoot)
  const groups = await Promise.all(
    monthDirs.map(async (month) => {
      const ids = await idsInDirectory(join(archiveRoot, month))
      return Promise.all(
        ids.map(async (id) => ({
          ticket: await readTicketFile(join(archiveRoot, month, `${id}.json`)),
          sourceArchiveMonth: month,
        })),
      )
    }),
  )
  return groups.flat()
}

async function archiveMonthDirs(archiveRoot: string): Promise<readonly string[]> {
  try {
    const entries = await readdir(archiveRoot, { withFileTypes: true })
    return entries
      .filter((entry) => entry.isDirectory() && /^\d{4}-\d{2}$/.test(entry.name))
      .map((entry) => entry.name)
      .sort()
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return []
    }
    throw error
  }
}

function rejectDuplicateSources(sourceTickets: readonly SourceTicket[]): void {
  const seen = new Set<string>()
  for (const sourceTicket of sourceTickets) {
    if (seen.has(sourceTicket.ticket.id)) {
      throw new DuplicateSourceTicketError(sourceTicket.ticket.id)
    }
    seen.add(sourceTicket.ticket.id)
  }
}

async function rejectDestinationCollisions(
  destination: TicketStorePaths,
  sourceTickets: readonly SourceTicket[],
): Promise<void> {
  const destinationIds = await destinationTicketIds(destination)
  for (const sourceTicket of sourceTickets) {
    if (destinationIds.has(sourceTicket.ticket.id)) {
      throw new DuplicateDestinationTicketError(sourceTicket.ticket.id)
    }
  }
}

async function destinationTicketIds(destination: TicketStorePaths): Promise<ReadonlySet<string>> {
  const activeIds = await idsInDirectory(destination.active)
  const monthDirs = await archiveMonthDirs(destination.archive)
  const archiveIdGroups = await Promise.all(
    monthDirs.map((month) => idsInDirectory(join(destination.archive, month))),
  )
  return new Set([...activeIds, ...archiveIdGroups.flat()])
}

async function writeArchivedTicket(
  destination: TicketStorePaths,
  sourceTicket: SourceTicket,
): Promise<void> {
  const month = archiveMonth(sourceTicket)
  const file = join(destination.archive, month, `${sourceTicket.ticket.id}.json`)
  await mkdir(dirname(file), { recursive: true })
  await writeJson(file, sourceTicket.ticket)
}

function archiveMonth(sourceTicket: SourceTicket): string {
  return (
    sourceTicket.ticket.closed?.slice(0, 7) ??
    sourceTicket.sourceArchiveMonth ??
    sourceTicket.ticket.updated.slice(0, 7)
  )
}
