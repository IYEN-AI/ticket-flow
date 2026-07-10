import { stat, writeFile } from "node:fs/promises"

type TicketFixtureOverrides = {
  readonly id: string
  readonly title: string
  readonly status: "open" | "doing" | "review" | "blocked" | "done"
  readonly updated: string
  readonly closed?: string | null
}

export async function exists(path: string): Promise<boolean> {
  try {
    await stat(path)
    return true
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false
    }
    throw error
  }
}

export async function writeTicketFixture(
  path: string,
  overrides: TicketFixtureOverrides,
): Promise<void> {
  await writeFile(
    path,
    `${JSON.stringify({
      id: overrides.id,
      title: overrides.title,
      status: overrides.status,
      priority: "medium",
      type: "chore",
      source: null,
      goal: "",
      acceptance: [],
      tags: [],
      artifacts: [],
      links: { github_issues: [], prs: [], threads: [], cron_jobs: [] },
      parent: null,
      children: [],
      assignee: "iyen",
      created: "2026-06-01T00:00:00Z",
      updated: overrides.updated,
      closed: overrides.closed ?? null,
      log: [],
    })}\n`,
  )
}
