import { emitTicketEventFromEnv } from "../clawhip"
import { InvalidSourceError, TicketNotFoundError } from "../errors"
import { type CreateTicketInput, CreateTicketInputSchema, type Ticket } from "../schema"
import { nextTicketId } from "./id"
import { readActiveTicket, upsertIndex, writeTicket } from "./io"
import type { TicketStorePaths } from "./paths"
import { ensureStore, resolveStorePaths } from "./paths"
import { utcNow } from "./time"

export async function createTicket(
  inputValue: CreateTicketInput,
  paths = resolveStorePaths(),
): Promise<Ticket> {
  await ensureStore(paths)
  const input = CreateTicketInputSchema.parse(inputValue)
  const id = await nextTicketId(paths)
  const now = utcNow()
  const ticket: Ticket = {
    id,
    title: input.title,
    status: "open",
    priority: input.priority,
    type: input.type,
    source: parseSource(input.source),
    goal: input.goal,
    acceptance: input.acceptance.map((item) => item.trim()).filter((item) => item.length > 0),
    tags: input.tags.map((item) => item.trim()).filter((item) => item.length > 0),
    artifacts: [],
    links: { github_issues: [], prs: [], threads: [], cron_jobs: [] },
    parent: input.parent ?? null,
    children: [],
    assignee: input.assignee,
    created: now,
    updated: now,
    closed: null,
    log: [{ ts: now, action: "created", note: "Ticket created" }],
  }
  await writeTicket(paths, ticket)
  if (ticket.parent !== null) {
    await appendChild(paths, ticket.parent, ticket.id, now)
  }
  await upsertIndex(paths, ticket)
  await emitTicketEventFromEnv({ kind: "ticket.created", ticket })
  return ticket
}

async function appendChild(
  paths: TicketStorePaths,
  parentId: string,
  childId: string,
  now: string,
): Promise<void> {
  try {
    const parent = await readActiveTicket(paths, parentId)
    if (parent.children.includes(childId)) {
      return
    }
    await writeTicket(paths, { ...parent, children: [...parent.children, childId], updated: now })
  } catch (error) {
    if (error instanceof TicketNotFoundError) {
      return
    }
    throw error
  }
}

function parseSource(source?: string): Ticket["source"] {
  if (source === undefined || source.trim().length === 0) {
    return null
  }
  const separator = source.indexOf(":")
  if (separator <= 0 || separator === source.length - 1) {
    throw new InvalidSourceError(source)
  }
  return { type: source.slice(0, separator), ref: source.slice(separator + 1) }
}
