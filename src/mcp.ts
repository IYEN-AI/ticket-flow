#!/usr/bin/env bun
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js"
import { z } from "zod"
import {
  LinkKindSchema,
  NextActionTypeSchema,
  TicketPrioritySchema,
  TicketStatusSchema,
} from "./schema"
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

const server = new McpServer({ name: "ticket-flow", version: "0.1.0" })

server.registerTool(
  "ticket_create",
  {
    description: "Create an agent ticket in the configured local ticket store.",
    inputSchema: {
      title: z.string().min(1),
      type: z.string().min(1).default("chore"),
      priority: TicketPrioritySchema.default("medium"),
      goal: z.string().default(""),
      assignee: z.string().min(1).default("iyen"),
      acceptance: z.array(z.string()).default([]),
      tags: z.array(z.string()).default([]),
      parent: z.string().optional(),
      source: z.string().optional(),
    },
  },
  async (input) => {
    const ticket = await createTicket(input)
    return {
      content: [{ type: "text", text: JSON.stringify(ticket, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_list",
  {
    description: "List active agent tickets from active JSON files.",
    inputSchema: { status: TicketStatusSchema.optional() },
  },
  async ({ status }) => {
    const tickets = await listTickets(undefined, status)
    return {
      content: [{ type: "text", text: JSON.stringify(tickets, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_get",
  {
    description: "Read an active or archived agent ticket by ID.",
    inputSchema: { id: z.string().regex(/^T-\d{8}-\d{3}$/) },
  },
  async ({ id }) => {
    const ticket = await getTicket(id)
    return {
      content: [{ type: "text", text: JSON.stringify(ticket, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_update_status",
  {
    description: "Apply the ticket status transition rules.",
    inputSchema: {
      id: z.string().regex(/^T-\d{8}-\d{3}$/),
      status: TicketStatusSchema,
      artifact: z.string().optional(),
      evidence: z.string().optional(),
      note: z.string().optional(),
    },
  },
  async (input) => {
    const result = await updateStatus(input)
    return {
      content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_link",
  {
    description: "Attach a GitHub issue, PR, Discord thread, or cron job link to an active ticket.",
    inputSchema: {
      id: z.string().regex(/^T-\d{8}-\d{3}$/),
      kind: LinkKindSchema,
      value: z.string().min(1),
    },
  },
  async (input) => {
    const ticket = await linkTicket(input)
    return {
      content: [{ type: "text", text: JSON.stringify(ticket, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_add_log",
  {
    description: "Append a note log entry to an active ticket.",
    inputSchema: {
      id: z.string().regex(/^T-\d{8}-\d{3}$/),
      note: z.string().min(1),
    },
  },
  async (input) => {
    const ticket = await addLog(input)
    return {
      content: [{ type: "text", text: JSON.stringify(ticket, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_checkpoint",
  {
    description: "Write the checkpoint/current handoff payload to an active ticket.",
    inputSchema: {
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
    },
  },
  async (input) => {
    const ticket = await checkpointTicket(input)
    return {
      content: [{ type: "text", text: JSON.stringify(ticket, null, 2) }],
    }
  },
)

server.registerTool(
  "ticket_agent_actions",
  {
    description: "List active tickets whose current.next_action.type is agent_action.",
    inputSchema: { staleMinutes: z.number().int().nonnegative().optional() },
  },
  async ({ staleMinutes }) => {
    const actions = await listAgentActions(undefined, staleMinutes)
    return {
      content: [{ type: "text", text: JSON.stringify(actions, null, 2) }],
    }
  },
)

server.registerResource(
  "active-tickets",
  "tickets://active",
  {
    title: "Active agent tickets",
    description: "All active tickets as JSON.",
    mimeType: "application/json",
  },
  async (uri) => {
    const tickets = await listTickets()
    return {
      contents: [
        { uri: uri.href, mimeType: "application/json", text: JSON.stringify(tickets, null, 2) },
      ],
    }
  },
)

const transport = new StdioServerTransport()
await server.connect(transport)
