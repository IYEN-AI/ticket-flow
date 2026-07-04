import { mkdir, rm } from "node:fs/promises"
import { dirname, join } from "node:path"
import { emitTicketEventFromEnv } from "../clawhip"
import { InvalidTransitionError, ReviewArtifactRequiredError } from "../errors"
import { type StatusInput, StatusInputSchema, type Ticket, type TicketStatus } from "../schema"
import { readActiveTicket, removeIndex, upsertIndex, writeJson, writeTicket } from "./io"
import { ensureStore, resolveStorePaths } from "./paths"
import { utcNow } from "./time"

export async function updateStatus(
  inputValue: StatusInput,
  paths = resolveStorePaths(),
): Promise<{
  readonly id: string
  readonly oldStatus: TicketStatus
  readonly newStatus: TicketStatus
}> {
  await ensureStore(paths)
  const input = StatusInputSchema.parse(inputValue)
  const ticket = await readActiveTicket(paths, input.id)
  if (!canTransition(ticket.status, input.status)) {
    throw new InvalidTransitionError(ticket.status, input.status)
  }
  if (input.status === "review" && input.artifact === undefined && ticket.artifacts.length === 0) {
    throw new ReviewArtifactRequiredError(ticket.id)
  }

  const now = utcNow()
  const oldStatus = ticket.status
  const nextTicket: Ticket = {
    ...ticket,
    status: input.status,
    updated: now,
    closed: input.status === "done" ? now : ticket.closed,
    artifacts: [...ticket.artifacts, ...artifactEntries(now, input.artifact, input.evidence)],
    log: [
      ...ticket.log,
      {
        ts: now,
        action: `${oldStatus} -> ${input.status}`,
        ...(input.note === undefined ? {} : { note: input.note }),
      },
    ],
  }

  if (input.status === "done") {
    const archiveFile = join(paths.archive, now.slice(0, 7), `${ticket.id}.json`)
    await mkdir(dirname(archiveFile), { recursive: true })
    await writeJson(archiveFile, nextTicket)
    await rm(join(paths.active, `${ticket.id}.json`))
    await removeIndex(paths, ticket.id)
  } else {
    await writeTicket(paths, nextTicket)
    await upsertIndex(paths, nextTicket)
  }

  await emitTicketEventFromEnv({
    kind: "ticket.status_changed",
    ticket: nextTicket,
    fromStatus: oldStatus,
    toStatus: input.status,
  })
  return { id: ticket.id, oldStatus, newStatus: input.status }
}

function artifactEntries(
  now: string,
  artifact?: string,
  evidence?: string,
): readonly Ticket["artifacts"][number][] {
  return [
    ...(artifact === undefined ? [] : [{ type: "artifact", value: artifact, ts: now }]),
    ...(evidence === undefined ? [] : [{ type: "evidence", value: evidence, ts: now }]),
  ]
}

function canTransition(from: TicketStatus, to: TicketStatus): boolean {
  switch (from) {
    case "open":
      return to === "doing" || to === "blocked"
    case "doing":
      return to === "review" || to === "blocked" || to === "open"
    case "review":
      return to === "done" || to === "open"
    case "blocked":
      return to === "doing" || to === "open"
    case "done":
      return false
  }
}
