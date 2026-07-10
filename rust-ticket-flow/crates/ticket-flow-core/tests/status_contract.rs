use tempfile::tempdir;
use ticket_flow_core::{CreateTicket, StatusPatch, StorePaths, TicketStatus, TicketStore};

#[test]
fn status_rejects_open_directly_to_done() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a fresh open ticket.
    let temp = tempdir()?;
    let store = TicketStore::new(StorePaths::new(temp.path().to_path_buf()));
    let ticket = store.create_ticket(CreateTicket::new("transition".to_owned()))?;

    // When: done is requested without passing through review.
    let result = store.update_status(
        &ticket.id,
        StatusPatch {
            status: TicketStatus::Done,
            artifact: None,
            evidence: None,
            note: None,
        },
    );

    // Then: the transition is rejected.
    let error = result.err().ok_or("expected invalid transition")?;
    assert!(
        error
            .to_string()
            .contains("invalid transition open -> done")
    );
    Ok(())
}

#[test]
fn status_persists_artifact_evidence_and_note() -> Result<(), Box<dyn std::error::Error>> {
    // Given: an open ticket.
    let temp = tempdir()?;
    let store = TicketStore::new(StorePaths::new(temp.path().to_path_buf()));
    let ticket = store.create_ticket(CreateTicket::new("status fields".to_owned()))?;

    // When: the ticket moves to doing with artifact, evidence, and note.
    let updated = store.update_status(
        &ticket.id,
        StatusPatch {
            status: TicketStatus::Doing,
            artifact: Some("artifact path".to_owned()),
            evidence: Some("test passed".to_owned()),
            note: Some("started".to_owned()),
        },
    )?;

    // Then: all status payload fields are persisted.
    assert_eq!(updated.artifacts[0].artifact_type, "artifact");
    assert_eq!(updated.artifacts[0].value, "artifact path");
    assert_eq!(updated.artifacts[1].artifact_type, "evidence");
    assert_eq!(updated.artifacts[1].value, "test passed");
    assert_eq!(updated.log[0].action, Some("open -> doing".to_owned()));
    assert_eq!(updated.log[0].note, Some("started".to_owned()));
    Ok(())
}

#[test]
fn review_uses_existing_artifact() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a doing ticket that already has an artifact.
    let temp = tempdir()?;
    let store = TicketStore::new(StorePaths::new(temp.path().to_path_buf()));
    let ticket = store.create_ticket(CreateTicket::new("review".to_owned()))?;
    let doing = store.update_status(
        &ticket.id,
        StatusPatch {
            status: TicketStatus::Doing,
            artifact: Some("existing artifact".to_owned()),
            evidence: None,
            note: None,
        },
    )?;

    // When: review is requested without a new artifact.
    let reviewed = store.update_status(
        &doing.id,
        StatusPatch {
            status: TicketStatus::Review,
            artifact: None,
            evidence: None,
            note: Some("ready".to_owned()),
        },
    )?;

    // Then: the existing artifact satisfies the review gate.
    assert_eq!(reviewed.status, TicketStatus::Review);
    assert_eq!(reviewed.artifacts.len(), 1);
    assert_eq!(reviewed.log[1].action, Some("doing -> review".to_owned()));
    Ok(())
}
