import { emitTicketEventFromEnv } from "../clawhip"
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
  await emitTicketEventFromEnv({ kind: "ticket.checkpointed", ticket: nextTicket })
  if (nextAction?.type === "agent_action") {
    await emitTicketEventFromEnv({ kind: "ticket.agent_action_available", ticket: nextTicket })
  }
  if (nextAction?.type === "blocked" || nextTicket.current?.blocker !== undefined) {
    await emitTicketEventFromEnv({ kind: "ticket.blocked", ticket: nextTicket })
  }
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
    type: input.nextType,
    command: resolveOptionalText(input.nextCommand),
    owner: resolveOptionalText(input.nextOwner),
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
    phase: resolveOptionalText(input.phase),
    decision: resolveOptionalText(input.decision),
    evidence: resolveOptionalText(input.evidence),
    blocker: resolveOptionalText(input.blocker),
    next: resolveOptionalText(input.next),
    note: resolveOptionalText(input.note),
    ...(nextAction === undefined ? {} : { next_action: nextAction }),
  }
}

function resolveOptionalText(value: string | undefined): string | undefined {
  return value === undefined || value.length === 0 ? undefined : value
}
