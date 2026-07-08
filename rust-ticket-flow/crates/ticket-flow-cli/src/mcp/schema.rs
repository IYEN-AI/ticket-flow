use serde_json::{Value, json};

pub(crate) fn tool_definitions() -> Value {
    json!([
        tool(
            "ticket_create",
            "Create an agent ticket.",
            object(
                json!({
                    "title": string("Ticket title"),
                    "type": string("Ticket type"),
                    "priority": string("Ticket priority"),
                    "goal": string("Goal statement"),
                    "assignee": string("Ticket owner"),
                    "acceptance": string_array("Acceptance criteria"),
                    "tags": string_array("Tags"),
                    "parent": string("Parent ticket ID"),
                    "source": string("Source reference as kind:ref")
                }),
                &["title"],
            )
        ),
        tool(
            "ticket_list",
            "List active agent tickets.",
            object(json!({"status": status_enum()}), &[],)
        ),
        tool(
            "ticket_get",
            "Read an active or archived ticket.",
            id_schema()
        ),
        tool(
            "ticket_import",
            "Import a source ticket store into ticket-flow.",
            object(
                json!({"sourceRoot": string("Source ticket store root")}),
                &["sourceRoot"],
            )
        ),
        tool(
            "ticket_update_status",
            "Apply a ticket status transition.",
            object(
                json!({
                    "id": string("Ticket ID"),
                    "status": status_enum(),
                    "artifact": string("Artifact value for review/done transitions"),
                    "evidence": string("Evidence artifact value"),
                    "note": string("Status transition note")
                }),
                &["id", "status"],
            )
        ),
        tool(
            "ticket_link",
            "Attach an external reference to a ticket.",
            object(
                json!({
                    "id": string("Ticket ID"),
                    "kind": {"type":"string","enum":["github_issues","prs","threads","cron_jobs"]},
                    "value": string("Reference value")
                }),
                &["id", "kind", "value"],
            )
        ),
        tool(
            "ticket_add_log",
            "Append a note log entry to a ticket.",
            object(
                json!({"id": string("Ticket ID"), "note": string("Log note")}),
                &["id", "note"],
            )
        ),
        tool(
            "ticket_checkpoint",
            "Write the current checkpoint payload.",
            object(
                json!({
                    "id": string("Ticket ID"),
                    "phase": string("Current phase"),
                    "decision": string("Current decision"),
                    "evidence": string("Evidence summary"),
                    "blocker": string("Blocker summary"),
                    "next": string("Next step summary"),
                    "note": string("Checkpoint note"),
                    "nextType": {"type":"string","enum":["agent_action","owner_gate","release_gate","blocked"]},
                    "nextCommand": string("Agent command"),
                    "nextOwner": string("Next owner")
                }),
                &["id"],
            )
        ),
        tool(
            "ticket_agent_actions",
            "List tickets with agent_action next actions.",
            object(json!({}), &[],)
        )
    ])
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": input_schema })
}

fn id_schema() -> Value {
    object(json!({"id": string("Ticket ID")}), &["id"])
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn string(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn string_array(description: &str) -> Value {
    json!({ "type": "array", "items": {"type":"string"}, "description": description })
}

fn status_enum() -> Value {
    json!({ "type":"string", "enum":["open","doing","review","blocked","done"] })
}
