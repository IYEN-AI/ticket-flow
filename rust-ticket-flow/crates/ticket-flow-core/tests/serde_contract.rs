use serde_json::json;
use ticket_flow_core::{EvidenceStatus, Ticket, TicketId, TicketIndex, TicketStatus};

#[test]
fn legacy_ticket_defaults_and_unknown_fields_round_trip() {
    // Given: a legacy ticket with omitted optional fields and one future field.
    let ticket_json = json!({
        "id": "T-20260708-001",
        "title": "legacy",
        "status": "open",
        "priority": "medium",
        "type": "chore",
        "created": "2026-07-08T00:00:00Z",
        "updated": "2026-07-08T00:00:00Z",
        "future_field": {"keep": true}
    });

    // When: Rust parses and serializes it.
    let ticket: Ticket = serde_json::from_value(ticket_json).expect("parse legacy ticket");
    let round_trip = serde_json::to_value(&ticket).expect("serialize ticket");

    // Then: defaults are filled and unknown fields survive.
    assert_eq!(ticket.links.github_issues, Vec::<String>::new());
    assert_eq!(round_trip["future_field"], json!({"keep": true}));
}

#[test]
fn legacy_string_artifact_normalizes_to_object() {
    // Given: TypeScript-compatible legacy string artifacts.
    let ticket_json = json!({
        "id": "T-20260708-002",
        "title": "legacy artifact",
        "status": "open",
        "priority": "medium",
        "type": "chore",
        "created": "2026-07-08T00:00:00Z",
        "updated": "2026-07-08T00:00:00Z",
        "artifacts": ["legacy artifact"]
    });

    // When: the ticket crosses the Rust serde boundary.
    let ticket: Ticket = serde_json::from_value(ticket_json).expect("parse artifact");

    // Then: the artifact is normalized to the object shape.
    let artifact = ticket.artifacts.first().expect("artifact exists");
    assert_eq!(artifact.artifact_type, "artifact");
    assert_eq!(artifact.value, "legacy artifact");
    assert_eq!(artifact.ts, "");
}

#[test]
fn index_summary_contract_matches_typescript() {
    // Given: a TypeScript index file.
    let index_json = json!({
        "version": 1,
        "lastId": "T-20260708-002",
        "tickets": {
            "T-20260708-002": {
                "title": "indexed",
                "status": "doing",
                "priority": "high",
                "type": "agent_action",
                "updated": "2026-07-08T00:00:00Z"
            }
        }
    });

    // When: Rust parses the index.
    let index: TicketIndex = serde_json::from_value(index_json).expect("parse index");

    // Then: the summary key and status remain typed.
    let id = TicketId::parse("T-20260708-002").expect("id parses");
    assert_eq!(index.last_id, Some(id.clone()));
    assert_eq!(
        index.tickets.get(&id).expect("summary").status,
        TicketStatus::Doing
    );
}

#[test]
fn v2_optional_sections_round_trip() {
    // Given: a v2 ticket with evidence and coordination sections.
    let ticket_json = json!({
        "id": "T-20260708-003",
        "title": "v2",
        "status": "open",
        "priority": "medium",
        "type": "chore",
        "created": "2026-07-08T00:00:00Z",
        "updated": "2026-07-08T00:00:00Z",
        "evidence": {"evidence_status": "partial", "artifacts": []},
        "coordination": {"owners": ["codex"], "handoffs": [], "approvals": [], "blockers": []}
    });

    // When: Rust parses and serializes it.
    let ticket: Ticket = serde_json::from_value(ticket_json).expect("parse v2 ticket");
    let serialized = serde_json::to_value(&ticket).expect("serialize v2 ticket");

    // Then: optional sections can be present and round-trip.
    assert_eq!(
        ticket.evidence.expect("evidence").evidence_status,
        EvidenceStatus::Partial
    );
    assert_eq!(serialized["coordination"]["owners"], json!(["codex"]));
}

#[test]
fn rejects_invalid_ticket_id_and_status() {
    // Given: invalid boundary values.
    let bad_id = json!({
        "id": "bad",
        "title": "bad",
        "status": "open",
        "priority": "medium",
        "type": "chore",
        "created": "2026-07-08T00:00:00Z",
        "updated": "2026-07-08T00:00:00Z"
    });
    let bad_status = json!({
        "id": "T-20260708-004",
        "title": "bad",
        "status": "bogus",
        "priority": "medium",
        "type": "chore",
        "created": "2026-07-08T00:00:00Z",
        "updated": "2026-07-08T00:00:00Z"
    });

    // When/Then: serde rejects them at the boundary.
    assert!(serde_json::from_value::<Ticket>(bad_id).is_err());
    assert!(serde_json::from_value::<Ticket>(bad_status).is_err());
}
