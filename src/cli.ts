#!/usr/bin/env bun
import { Command } from "commander"
import { buildClawhipEvent, ClawhipEventKindSchema, sendClawhipEvent } from "./clawhip"
import {
  type AgentActionsOptions,
  type CheckpointOptions,
  type ClawhipEventOptions,
  type CreateOptions,
  collect,
  collectTags,
  type LinkOptions,
  type ListOptions,
  resolveLinkOption,
  resolveSourceOption,
  runBoundary,
  type StatusOptions,
} from "./cli-support"
import { NextActionTypeSchema, TicketStatusSchema } from "./schema"
import {
  addLog,
  checkpointTicket,
  createTicket,
  getTicket,
  linkTicket,
  listAgentActions,
  listTickets,
  updateStatus,
} from "./store"

const program = new Command()

program.name("ticket-flow").description("OpenClaw-compatible ticket CLI").version("0.1.0")

program
  .command("create")
  .requiredOption("--title <title>")
  .option("--type <type>", "ticket type", "chore")
  .option("--priority <priority>", "priority", "medium")
  .option("--goal <goal>", "goal", "")
  .option("--parent <id>")
  .option("--assignee <assignee>", "assignee", "iyen")
  .option("--acceptance <acceptance>", "acceptance criterion", collect, [])
  .option("--tag <tags>", "comma-separated tags", collectTags, [])
  .option("--source <source...>", "source as type:ref or type ref")
  .action(async (options: CreateOptions) => {
    await runBoundary(async () => {
      const ticket = await createTicket({
        title: options.title,
        type: options.type,
        priority: options.priority,
        goal: options.goal,
        parent: options.parent,
        assignee: options.assignee,
        acceptance: [...options.acceptance],
        tags: [...options.tag],
        source: resolveSourceOption(options.source),
      })
      console.log(ticket.id)
    })
  })

program
  .command("status")
  .argument("<id>")
  .argument("<status>")
  .option("--artifact <artifact>")
  .option("--evidence <evidence>")
  .option("--note <note>")
  .action(async (id: string, statusValue: string, options: StatusOptions) => {
    await runBoundary(async () => {
      const status = TicketStatusSchema.parse(statusValue)
      const result = await updateStatus({
        id,
        status,
        artifact: options.artifact,
        evidence: options.evidence,
        note: options.note,
      })
      console.log(`${result.id}: ${result.oldStatus} -> ${result.newStatus}`)
    })
  })

program
  .command("list")
  .option("--status <status>")
  .action(async (options: ListOptions) => {
    await runBoundary(async () => {
      const status =
        options.status === undefined ? undefined : TicketStatusSchema.parse(options.status)
      const tickets = await listTickets(undefined, status)
      if (tickets.length === 0) {
        console.log("No active tickets.")
        return
      }
      for (const ticket of tickets) {
        console.log(`  ${ticket.id}  [${ticket.status}] [${ticket.priority}] ${ticket.title}`)
      }
    })
  })

program
  .command("show")
  .argument("<id>")
  .action(async (id: string) => {
    await runBoundary(async () => {
      const ticket = await getTicket(id)
      console.log(JSON.stringify(ticket, null, 2))
    })
  })

program
  .command("link")
  .argument("<id>")
  .option("--github-issue <value>")
  .option("--pr <value>")
  .option("--thread <value>")
  .option("--cron <value>")
  .action(async (id: string, options: LinkOptions) => {
    await runBoundary(async () => {
      const link = resolveLinkOption(options)
      const ticket = await linkTicket({ id, kind: link.kind, value: link.value })
      console.log(`${ticket.id}: linked ${link.kind} -> ${link.value}`)
    })
  })

program
  .command("log")
  .argument("<id>")
  .argument("<note...>")
  .action(async (id: string, note: string[]) => {
    await runBoundary(async () => {
      const ticket = await addLog({ id, note: note.join(" ") })
      console.log(`${ticket.id}: logged`)
    })
  })

program
  .command("checkpoint")
  .argument("<id>")
  .option("--phase <phase>")
  .option("--decision <decision>")
  .option("--evidence <evidence>")
  .option("--blocker <blocker>")
  .option("--next <next>")
  .option("--next-type <nextType>")
  .option("--next-command <nextCommand>")
  .option("--next-owner <nextOwner>")
  .option("--note <note>")
  .action(async (id: string, options: CheckpointOptions) => {
    await runBoundary(async () => {
      const ticket = await checkpointTicket({
        id,
        phase: options.phase,
        decision: options.decision,
        evidence: options.evidence,
        blocker: options.blocker,
        next: options.next,
        nextType:
          options.nextType === undefined ? undefined : NextTypeOptionSchema.parse(options.nextType),
        nextCommand: options.nextCommand,
        nextOwner: options.nextOwner,
        note: options.note,
      })
      console.log(`${ticket.id}: checkpoint logged`)
    })
  })

program
  .command("agent-actions")
  .option("--stale-minutes <minutes>")
  .action(async (options: AgentActionsOptions) => {
    await runBoundary(async () => {
      const staleMinutes =
        options.staleMinutes === undefined ? undefined : Number.parseInt(options.staleMinutes, 10)
      const actions = await listAgentActions(undefined, staleMinutes)
      for (const action of actions) {
        const age = action.ageMinutes === undefined ? "" : ` age=${action.ageMinutes}m`
        console.log(
          `${action.id} [${action.status}] owner=${action.owner}${age} :: ${action.command}`,
        )
      }
    })
  })

const clawhip = program.command("clawhip").description("clawhip integration helpers")

clawhip
  .command("event")
  .argument("<id>")
  .requiredOption("--kind <kind>")
  .option("--print", "print the clawhip IncomingEvent JSON")
  .option("--send", "send the event to the clawhip daemon")
  .option("--url <url>", "clawhip daemon base URL", "http://127.0.0.1:25294")
  .option("--timeout-ms <timeoutMs>", "clawhip send timeout in milliseconds", "1000")
  .action(async (id: string, options: ClawhipEventOptions) => {
    await runBoundary(async () => {
      const ticket = await getTicket(id)
      const kind = ClawhipEventKindSchema.parse(options.kind)
      const event = buildClawhipEvent({
        kind,
        ticket,
        repoPath: Bun.env["TICKET_FLOW_REPO_PATH"] ?? process.cwd(),
        worktreePath: Bun.env["TICKET_FLOW_WORKTREE_PATH"],
      })
      if (options.print === true || options.send !== true) {
        console.log(JSON.stringify(event, null, 2))
      }
      if (options.send === true) {
        const result = await sendClawhipEvent({
          url: options.url,
          event,
          mode: "strict",
          timeoutMs: Number.parseInt(options.timeoutMs ?? "1000", 10),
        })
        if (!result.ok) {
          throw new Error(result.error)
        }
        console.error(`clawhip: sent ${event.type} status=${result.status}`)
      }
    })
  })

const NextTypeOptionSchema = NextActionTypeSchema

await program.parseAsync()
