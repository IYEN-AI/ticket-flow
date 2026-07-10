use tempfile::tempdir;
use ticket_flow_core::{
    CheckpointPatch, ContextAudience, CreateTicket, EvidenceInput, NextActionType, StorePaths,
    TicketStore, attach_evidence, build_context_pack, ready_gate,
};

#[test]
fn checkpoint_requires_at_least_one_field() {
    // Given: a new ticket in a temporary store.
    let temp = tempdir().expect("tempdir");
    let store = TicketStore::new(StorePaths::new(temp.path().to_path_buf()));
    let ticket = store
        .create_ticket(CreateTicket::new("empty checkpoint".to_owned()))
        .expect("create ticket");

    // When: an empty checkpoint is submitted.
    let result = store.checkpoint_ticket(&ticket.id, CheckpointPatch::default());

    // Then: it is rejected.
    assert!(result.is_err());
}

#[test]
fn ready_gate_reports_missing_goal_acceptance_next_action() {
    // Given: a ticket with no goal, acceptance, or next action.
    let temp = tempdir().expect("tempdir");
    let store = TicketStore::new(StorePaths::new(temp.path().to_path_buf()));
    let ticket = store
        .create_ticket(CreateTicket::new("missing ready data".to_owned()))
        .expect("create ticket");

    // When: the ready gate runs.
    let result = ready_gate(&ticket);

    // Then: the missing fields are explicit.
    assert!(!result.passed);
    assert!(result.missing.contains(&"goal".to_owned()));
    assert!(result.missing.contains(&"acceptance".to_owned()));
    assert!(result.missing.contains(&"current.next_action".to_owned()));
}

#[test]
fn agent_context_pack_contains_goal_constraints_next_action_evidence() {
    // Given: a ticket with enough agent execution context.
    let temp = tempdir().expect("tempdir");
    let store = TicketStore::new(StorePaths::new(temp.path().to_path_buf()));
    let mut input = CreateTicket::new("agent context".to_owned());
    input.goal = "ship ticket-flow parity".to_owned();
    input.acceptance = vec!["agent queue has command".to_owned()];
    let ticket = store.create_ticket(input).expect("create ticket");
    store
        .checkpoint_ticket(
            &ticket.id,
            CheckpointPatch {
                phase: Some("implement".to_owned()),
                next_action_type: Some(NextActionType::AgentAction),
                next_command: Some("cargo test".to_owned()),
                next_owner: Some("codex".to_owned()),
                ..CheckpointPatch::default()
            },
        )
        .expect("checkpoint");
    attach_evidence(
        &store,
        &ticket.id,
        EvidenceInput {
            artifact_type: "test_output".to_owned(),
            value: "cargo test pass".to_owned(),
        },
    )
    .expect("attach evidence");

    // When: an agent context pack is built.
    let pack =
        build_context_pack(&store, &ticket.id, ContextAudience::AgentExecution).expect("context");

    // Then: the pack carries the working state.
    assert_eq!(pack.goal, "ship ticket-flow parity");
    assert_eq!(
        pack.next_action.expect("next action").command,
        Some("cargo test".to_owned())
    );
    assert_eq!(pack.evidence_refs, vec!["cargo test pass".to_owned()]);
}
