import { z } from "zod"

export const TicketStatusSchema = z.enum(["open", "doing", "review", "blocked", "done"])
export type TicketStatus = z.infer<typeof TicketStatusSchema>

export const TicketPrioritySchema = z.string().min(1)
export type TicketPriority = z.infer<typeof TicketPrioritySchema>

export const NextActionTypeSchema = z.enum([
  "agent_action",
  "owner_gate",
  "release_gate",
  "blocked",
])

export const SourceSchema = z.record(z.string(), z.unknown()).nullable().default(null)

export const ArtifactSchema = z.union([
  z
    .object({
      type: z.string().default("artifact"),
      value: z.string().default(""),
      ts: z.string().default(""),
    })
    .passthrough(),
  z.string().transform((value) => ({ type: "artifact", value, ts: "" })),
])

export const LinkMapSchema = z
  .object({
    github_issues: z.array(z.string()).default([]),
    prs: z.array(z.string()).default([]),
    threads: z.array(z.string()).default([]),
    cron_jobs: z.array(z.string()).default([]),
  })
  .default({ github_issues: [], prs: [], threads: [], cron_jobs: [] })
export const LinkKindSchema = z.enum(["github_issues", "prs", "threads", "cron_jobs"])
export type LinkKind = z.infer<typeof LinkKindSchema>

export const LogEntrySchema = z
  .object({
    ts: z.string().default(""),
    action: z.string().optional(),
    note: z.string().optional(),
    phase: z.string().optional(),
    decision: z.string().optional(),
    evidence: z.string().optional(),
    blocker: z.string().optional(),
    next: z.string().optional(),
    next_action: z
      .object({
        type: NextActionTypeSchema.optional(),
        command: z.string().optional(),
        owner: z.string().optional(),
      })
      .optional(),
  })
  .passthrough()

export const CurrentSchema = z
  .object({
    phase: z.string().optional(),
    decision: z.string().optional(),
    evidence: z.string().optional(),
    blocker: z.string().optional(),
    next: z.string().optional(),
    note: z.string().optional(),
    next_action: z
      .object({
        type: NextActionTypeSchema.optional(),
        command: z.string().optional(),
        owner: z.string().optional(),
      })
      .optional(),
  })
  .passthrough()

export const TicketSchema = z
  .object({
    id: z.string().regex(/^T-\d{8}-\d{3}$/),
    title: z.string().min(1),
    status: TicketStatusSchema,
    priority: TicketPrioritySchema,
    type: z.string().min(1),
    source: SourceSchema,
    goal: z.string().default(""),
    acceptance: z.array(z.string()).default([]),
    tags: z.array(z.string()).default([]),
    artifacts: z.array(ArtifactSchema).default([]),
    links: LinkMapSchema,
    parent: z.string().nullable().default(null),
    children: z.array(z.string()).default([]),
    assignee: z.string().default("iyen"),
    created: z.string(),
    updated: z.string(),
    closed: z.string().nullable().default(null),
    log: z.array(LogEntrySchema).default([]),
    current: CurrentSchema.optional(),
  })
  .passthrough()
export type Ticket = z.infer<typeof TicketSchema>

export const IndexSchema = z
  .object({
    version: z.number().optional(),
    lastId: z.string().nullable().optional(),
    tickets: z.record(
      z.string(),
      z
        .object({
          title: z.string(),
          status: TicketStatusSchema,
          priority: TicketPrioritySchema,
          type: z.string(),
          updated: z.string(),
        })
        .passthrough(),
    ),
  })
  .passthrough()
export type TicketIndex = z.infer<typeof IndexSchema>

export const CreateTicketInputSchema = z.object({
  title: z.string().min(1),
  type: z.string().min(1).default("chore"),
  priority: TicketPrioritySchema.default("medium"),
  goal: z.string().default(""),
  parent: z.string().optional(),
  assignee: z.string().min(1).default("iyen"),
  acceptance: z.array(z.string()).default([]),
  tags: z.array(z.string()).default([]),
  source: z.string().optional(),
})
export type CreateTicketInput = z.infer<typeof CreateTicketInputSchema>

export const StatusInputSchema = z.object({
  id: z.string().regex(/^T-\d{8}-\d{3}$/),
  status: TicketStatusSchema.exclude(["done"]).or(z.literal("done")),
  artifact: z.string().optional(),
  evidence: z.string().optional(),
  note: z.string().optional(),
})
export type StatusInput = z.infer<typeof StatusInputSchema>

export const CheckpointInputSchema = z
  .object({
    id: z.string().regex(/^T-\d{8}-\d{3}$/),
    phase: z.string().optional(),
    decision: z.string().optional(),
    evidence: z.string().optional(),
    blocker: z.string().optional(),
    next: z.string().optional(),
    note: z.string().optional(),
    nextType: NextActionTypeSchema.optional(),
    nextCommand: z.string().optional(),
    nextOwner: z.string().optional(),
  })
  .refine(
    (input) =>
      input.phase !== undefined ||
      input.decision !== undefined ||
      input.evidence !== undefined ||
      input.blocker !== undefined ||
      input.next !== undefined ||
      input.note !== undefined ||
      input.nextType !== undefined ||
      input.nextCommand !== undefined ||
      input.nextOwner !== undefined,
    { message: "checkpoint requires at least one field" },
  )
export type CheckpointInput = z.infer<typeof CheckpointInputSchema>

export const LinkInputSchema = z.object({
  id: z.string().regex(/^T-\d{8}-\d{3}$/),
  kind: LinkKindSchema,
  value: z.string().min(1),
})
export type LinkInput = z.infer<typeof LinkInputSchema>

export const AddLogInputSchema = z.object({
  id: z.string().regex(/^T-\d{8}-\d{3}$/),
  note: z.string().min(1),
})
export type AddLogInput = z.infer<typeof AddLogInputSchema>
