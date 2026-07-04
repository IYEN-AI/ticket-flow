import type { Ticket, TicketStatus } from "../schema"
import { type ClawhipSendMode, type ClawhipSendResult, sendClawhipEvent } from "./delivery"
import { buildClawhipEvent, type ClawhipEventKind } from "./events"

export type EmitTicketEventInput = {
  readonly kind: ClawhipEventKind
  readonly ticket: Ticket
  readonly fromStatus?: TicketStatus
  readonly toStatus?: TicketStatus
}

export async function emitTicketEventFromEnv(
  input: EmitTicketEventInput,
): Promise<ClawhipSendResult | undefined> {
  const config = resolveClawhipEnv()
  if (config === undefined) {
    return undefined
  }
  return sendClawhipEvent({
    url: config.url,
    mode: config.mode,
    timeoutMs: config.timeoutMs,
    event: buildClawhipEvent({
      kind: input.kind,
      ticket: input.ticket,
      fromStatus: input.fromStatus,
      toStatus: input.toStatus,
      repoPath: config.repoPath,
      worktreePath: config.worktreePath,
    }),
  })
}

type ClawhipEnvConfig = {
  readonly url: string
  readonly mode: ClawhipSendMode
  readonly timeoutMs: number
  readonly repoPath?: string | undefined
  readonly worktreePath?: string | undefined
}

function resolveClawhipEnv(): ClawhipEnvConfig | undefined {
  if (!isEnabled(Bun.env["TICKET_FLOW_CLAWHIP"])) {
    return undefined
  }
  return {
    url: Bun.env["TICKET_FLOW_CLAWHIP_URL"] ?? "http://127.0.0.1:25294",
    mode: Bun.env["TICKET_FLOW_CLAWHIP_MODE"] === "strict" ? "strict" : "best-effort",
    timeoutMs: parseTimeout(Bun.env["TICKET_FLOW_CLAWHIP_TIMEOUT_MS"]),
    repoPath: Bun.env["TICKET_FLOW_REPO_PATH"],
    worktreePath: Bun.env["TICKET_FLOW_WORKTREE_PATH"],
  }
}

function isEnabled(value: string | undefined): boolean {
  return value === "1" || value === "true" || value === "yes" || value === "on"
}

function parseTimeout(value: string | undefined): number {
  if (value === undefined) {
    return 1000
  }
  const parsed = Number.parseInt(value, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 1000
}
