import { InvalidTransitionError, ReviewArtifactRequiredError, TicketNotFoundError } from "./errors"
import type { LinkKind, TicketPriority, TicketStatus } from "./schema"

export type CreateOptions = {
  readonly title: string
  readonly type: string
  readonly priority: TicketPriority
  readonly goal: string
  readonly parent?: string
  readonly assignee: string
  readonly acceptance: readonly string[]
  readonly tag: readonly string[]
  readonly source?: string | readonly string[]
}

export type StatusOptions = {
  readonly artifact?: string
  readonly evidence?: string
  readonly note?: string
}

export type ListOptions = {
  readonly status?: TicketStatus
}

export type LinkOptions = {
  readonly githubIssue?: string
  readonly pr?: string
  readonly thread?: string
  readonly cron?: string
}

export type CheckpointOptions = {
  readonly phase?: string
  readonly decision?: string
  readonly evidence?: string
  readonly blocker?: string
  readonly next?: string
  readonly nextType?: string
  readonly nextCommand?: string
  readonly nextOwner?: string
  readonly note?: string
}

export type AgentActionsOptions = {
  readonly staleMinutes?: string
}

export type ClawhipEventOptions = {
  readonly kind: string
  readonly print?: boolean
  readonly send?: boolean
  readonly url?: string
  readonly timeoutMs?: string
}

export function collect(value: string, previous: readonly string[]): readonly string[] {
  return [...previous, value]
}

export function collectTags(value: string, previous: readonly string[]): readonly string[] {
  return [
    ...previous,
    ...value
      .split(",")
      .map((tag) => tag.trim())
      .filter((tag) => tag.length > 0),
  ]
}

export function resolveLinkOption(options: LinkOptions): {
  readonly kind: LinkKind
  readonly value: string
} {
  if (options.githubIssue !== undefined) {
    return { kind: "github_issues", value: options.githubIssue }
  }
  if (options.pr !== undefined) {
    return { kind: "prs", value: options.pr }
  }
  if (options.thread !== undefined) {
    return { kind: "threads", value: options.thread }
  }
  if (options.cron !== undefined) {
    return { kind: "cron_jobs", value: options.cron }
  }
  throw new Error("link requires --github-issue, --pr, --thread, or --cron")
}

export function resolveSourceOption(
  source: string | readonly string[] | undefined,
): string | undefined {
  if (source === undefined) {
    return undefined
  }
  if (typeof source === "string") {
    return source
  }
  if (source.length === 1) {
    return source[0]
  }
  const [type, ref, extra] = source
  if (type === undefined || ref === undefined || extra !== undefined) {
    throw new Error("--source must be either type:ref or type ref")
  }
  return `${type}:${ref}`
}

export async function runBoundary(work: () => Promise<void>): Promise<void> {
  try {
    await work()
  } catch (error) {
    if (error instanceof ReviewArtifactRequiredError) {
      console.error("Error: review requires --artifact")
      process.exitCode = 1
      return
    }
    if (error instanceof InvalidTransitionError || error instanceof TicketNotFoundError) {
      console.error(`Error: ${error.message}`)
      process.exitCode = 1
      return
    }
    if (error instanceof Error) {
      console.error(`Error: ${error.message}`)
      process.exitCode = 1
      return
    }
    throw error
  }
}
