import ky from "ky"
import type { ClawhipTicketEvent } from "./events"

const DEFAULT_CLAWHIP_URL = "http://127.0.0.1:25294"
const DEFAULT_TIMEOUT_MS = 1000

export type ClawhipSendMode = "best-effort" | "strict"

export type SendClawhipEventInput = {
  readonly url?: string | undefined
  readonly event: ClawhipTicketEvent
  readonly mode?: ClawhipSendMode
  readonly timeoutMs?: number
}

export type ClawhipSendResult =
  | {
      readonly ok: true
      readonly status: number
    }
  | {
      readonly ok: false
      readonly error: string
    }

export async function sendClawhipEvent(input: SendClawhipEventInput): Promise<ClawhipSendResult> {
  const mode = input.mode ?? "best-effort"
  try {
    const response = await ky.post(eventEndpoint(input.url ?? DEFAULT_CLAWHIP_URL), {
      json: input.event,
      retry: { limit: 0 },
      timeout: input.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    })
    return { ok: true, status: response.status }
  } catch (error) {
    if (mode === "strict") {
      throw error
    }
    if (error instanceof Error) {
      return { ok: false, error: error.message }
    }
    throw error
  }
}

function eventEndpoint(baseUrl: string): string {
  const url = new URL(baseUrl)
  if (url.pathname === "/" || url.pathname.length === 0) {
    url.pathname = "/event"
    return url.toString()
  }
  if (!url.pathname.endsWith("/event")) {
    url.pathname = `${url.pathname.replace(/\/$/, "")}/event`
  }
  return url.toString()
}
