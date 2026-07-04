import { basename } from "node:path"
import { z } from "zod"
import type { Ticket, TicketStatus } from "../schema"

export const ClawhipEventKindSchema = z.enum([
  "ticket.created",
  "ticket.status_changed",
  "ticket.checkpointed",
  "ticket.agent_action_available",
  "ticket.blocked",
  "ticket.review_ready",
])

export type ClawhipEventKind = z.infer<typeof ClawhipEventKindSchema>

export type ClawhipTicketEvent = {
  readonly type: ClawhipEventKind
  readonly payload: ClawhipTicketPayload
}

export type ClawhipTicketPayload = {
  readonly provider: "ticket-flow"
  readonly event: ClawhipEventKind
  readonly ticket_id: string
  readonly title: string
  readonly status: TicketStatus
  readonly priority?: string
  readonly type?: string
  readonly assignee?: string
  readonly from_status?: TicketStatus
  readonly to_status?: TicketStatus
  readonly phase?: string
  readonly decision?: string
  readonly blocker?: string
  readonly next?: string
  readonly next_action_type?: string
  readonly next_action_command?: string
  readonly next_action_owner?: string
  readonly summary: string
  readonly question_summary?: string
  readonly repo_path?: string
  readonly worktree_path?: string
  readonly repo_name?: string
  readonly event_timestamp: string
  readonly correlation_id?: string
}

export type BuildClawhipEventInput = {
  readonly kind: string
  readonly ticket: Ticket
  readonly repoPath?: string | undefined
  readonly worktreePath?: string | undefined
  readonly fromStatus?: TicketStatus | undefined
  readonly toStatus?: TicketStatus | undefined
  readonly correlationId?: string | undefined
}

export function buildClawhipEvent(input: BuildClawhipEventInput): ClawhipTicketEvent {
  const kind = ClawhipEventKindSchema.parse(input.kind)
  const repoPath = resolveText(input.repoPath)
  const worktreePath = resolveText(input.worktreePath) ?? repoPath
  const nextAction = input.ticket.current?.next_action
  const summary = `${input.ticket.id} [${input.ticket.status}] ${input.ticket.title}`
  return {
    type: kind,
    payload: {
      provider: "ticket-flow",
      event: kind,
      ticket_id: input.ticket.id,
      title: input.ticket.title,
      status: input.ticket.status,
      priority: input.ticket.priority,
      type: input.ticket.type,
      assignee: input.ticket.assignee,
      ...fromStatusField(input.fromStatus),
      ...toStatusField(input.toStatus),
      ...phaseField(input.ticket.current?.phase),
      ...decisionField(input.ticket.current?.decision),
      ...blockerField(input.ticket.current?.blocker),
      ...nextField(input.ticket.current?.next),
      ...nextActionTypeField(nextAction?.type),
      ...nextActionCommandField(nextAction?.command),
      ...nextActionOwnerField(nextAction?.owner),
      summary,
      ...questionSummaryField(questionSummary(kind, input.ticket)),
      ...repoPathField(repoPath),
      ...worktreePathField(worktreePath),
      ...repoNameField(repoPath === undefined ? undefined : basename(repoPath)),
      event_timestamp: input.ticket.updated,
      ...correlationIdField(input.correlationId),
    },
  }
}

function questionSummary(kind: ClawhipEventKind, ticket: Ticket): string | undefined {
  if (kind !== "ticket.blocked" && kind !== "ticket.review_ready") {
    return undefined
  }
  return ticket.current?.blocker ?? ticket.current?.next ?? ticket.title
}

function fromStatusField(
  value: TicketStatus | undefined,
): PickOptional<"from_status", TicketStatus> {
  return value === undefined ? {} : { from_status: value }
}

function toStatusField(value: TicketStatus | undefined): PickOptional<"to_status", TicketStatus> {
  return value === undefined ? {} : { to_status: value }
}

function phaseField(value: string | undefined): PickOptional<"phase", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { phase: trimmed }
}

function decisionField(value: string | undefined): PickOptional<"decision", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { decision: trimmed }
}

function blockerField(value: string | undefined): PickOptional<"blocker", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { blocker: trimmed }
}

function nextField(value: string | undefined): PickOptional<"next", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { next: trimmed }
}

function nextActionTypeField(value: string | undefined): PickOptional<"next_action_type", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { next_action_type: trimmed }
}

function nextActionCommandField(
  value: string | undefined,
): PickOptional<"next_action_command", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { next_action_command: trimmed }
}

function nextActionOwnerField(
  value: string | undefined,
): PickOptional<"next_action_owner", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { next_action_owner: trimmed }
}

function questionSummaryField(value: string | undefined): PickOptional<"question_summary", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { question_summary: trimmed }
}

function repoPathField(value: string | undefined): PickOptional<"repo_path", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { repo_path: trimmed }
}

function worktreePathField(value: string | undefined): PickOptional<"worktree_path", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { worktree_path: trimmed }
}

function repoNameField(value: string | undefined): PickOptional<"repo_name", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { repo_name: trimmed }
}

function correlationIdField(value: string | undefined): PickOptional<"correlation_id", string> {
  const trimmed = resolveText(value)
  return trimmed === undefined ? {} : { correlation_id: trimmed }
}

type PickOptional<K extends keyof ClawhipTicketPayload, V> = Partial<Readonly<Record<K, V>>>

function resolveText(value: string | undefined): string | undefined {
  if (value === undefined) {
    return undefined
  }
  const trimmed = value.trim()
  return trimmed.length === 0 ? undefined : trimmed
}
