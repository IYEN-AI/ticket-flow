import {
  type AddLogInput,
  AddLogInputSchema,
  type CheckpointInput,
  CheckpointInputSchema,
  type LinkInput,
  LinkInputSchema,
  type Ticket,
} from "../schema"
import { readActiveTicket, upsertIndex, writeTicket } from "./io"
import { ensureStore, resolveStorePaths } from "./paths"
import { utcNow } from "./time"

export async function linkTicket(
  inputValue: LinkInput,
  paths = resolveStorePaths(),
): Promise<Ticket> {
  await ensureStore(paths)
  const input = LinkInputSchema.parse(inputValue)
  const ticket = await readActiveTicket(paths, input.id)
  const existing = ticket.links[input.kind]
  const now = utcNow()
  const nextTicket: Ticket = {
    ...ticket,
    updated: now,
    links: {
      ...ticket.links,
      [input.kind]: existing.includes(input.value) ? existing : [...existing, input.value],
    },
  }
  await writeTicket(paths, nextTicket)
  await upsertIndex(paths, nextTicket)
  return nextTicket
}

export async function addLog(
  inputValue: AddLogInput,
  paths = resolveStorePaths(),
): Promise<Ticket> {
  await ensureStore(paths)
  const input = AddLogInputSchema.parse(inputValue)
  const ticket = await readActiveTicket(paths, input.id)
  const now = utcNow()
  const nextTicket: Ticket = {
    ...ticket,
    updated: now,
    log: [...ticket.log, { ts: now, action: "note", note: input.note }],
  }
  await writeTicket(paths, nextTicket)
  await upsertIndex(paths, nextTicket)
  return nextTicket
}

export async function checkpointTicket(
  inputValue: CheckpointInput,
  paths = resolveStorePaths(),
): Promise<Ticket> {
  await ensureStore(paths)
  const input = CheckpointInputSchema.parse(inputValue)
  const ticket = await readActiveTicket(paths, input.id)
  const now = utcNow()
  const nextAction = nextActionFromCheckpoint(input)
  const nextTicket: Ticket = {
    ...ticket,
    updated: now,
    current: checkpointCurrentPayload(nextAction, input),
    log: [...ticket.log, checkpointLogPayload(now, nextAction, input)],
  }
  await writeTicket(paths, nextTicket)
  await upsertIndex(paths, nextTicket)
  return nextTicket
}

type TicketNextAction = NonNullable<Ticket["current"]>["next_action"]

function nextActionFromCheckpoint(input: CheckpointInput): TicketNextAction {
  if (
    input.nextType === undefined &&
    input.nextCommand === undefined &&
    input.nextOwner === undefined
  ) {
    return undefined
  }
  return {
    ...(input.nextType === undefined ? {} : { type: input.nextType }),
    ...optionalText("command", input.nextCommand),
    ...optionalText("owner", input.nextOwner),
  }
}

function checkpointLogPayload(
  ts: string,
  nextAction: TicketNextAction,
  input: CheckpointInput,
): Ticket["log"][number] {
  return {
    ts,
    action: "checkpoint",
    ...checkpointCurrentPayload(nextAction, input),
  }
}

function checkpointCurrentPayload(
  nextAction: TicketNextAction,
  input: CheckpointInput,
): NonNullable<Ticket["current"]> {
  return {
    ...optionalText("phase", input.phase),
    ...optionalText("decision", input.decision),
    ...optionalText("evidence", input.evidence),
    ...optionalText("blocker", input.blocker),
    ...optionalText("next", input.next),
    ...optionalText("note", input.note),
    ...(nextAction === undefined ? {} : { next_action: nextAction }),
  }
}

function optionalText<K extends string>(
  key: K,
  value: string | undefined,
): Record<K, string> | object {
  return value === undefined || value.length === 0 ? {} : { [key]: value }
}
