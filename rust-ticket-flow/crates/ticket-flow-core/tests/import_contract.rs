use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::tempdir;
use ticket_flow_core::{StorePaths, TicketId, TicketIndex, TicketStore};

#[test]
fn import_store_copies_active_archived_index_events_and_leaves_source()
-> Result<(), Box<dyn std::error::Error>> {
    // Given: a source store with active and archived tickets.
    let source = tempdir()?;
    let destination = tempdir()?;
    write_ticket_fixture(
        &source.path().join("active").join("T-20260630-001.json"),
        json!({
            "id": "T-20260630-001",
            "title": "import active",
            "status": "open",
            "updated": "2026-06-30T00:00:00Z"
        }),
    )?;
    write_ticket_fixture(
        &source.path().join("active").join("T-20260630-002.json"),
        json!({
            "id": "T-20260630-002",
            "title": "import done from active",
            "status": "done",
            "closed": "2026-07-08T00:00:00Z",
            "updated": "2026-06-30T00:00:00Z"
        }),
    )?;
    write_ticket_fixture(
        &source
            .path()
            .join("archive")
            .join("2026-06")
            .join("T-20260629-001.json"),
        json!({
            "id": "T-20260629-001",
            "title": "import archived",
            "status": "done",
            "updated": "2026-06-29T00:00:00Z"
        }),
    )?;
    let store = TicketStore::new(StorePaths::new(destination.path().to_path_buf()));

    // When: the destination imports the source store.
    let summary = store.import_store(source.path())?;

    // Then: active tickets are indexed, done tickets are archived by month, and source files remain.
    assert_eq!(summary.imported, 3);
    assert_eq!(summary.active, 1);
    assert_eq!(summary.archived, 2);
    assert_file_exists(
        &destination
            .path()
            .join("active")
            .join("T-20260630-001.json"),
    );
    assert_file_exists(
        &destination
            .path()
            .join("archive")
            .join("2026-07")
            .join("T-20260630-002.json"),
    );
    assert_file_exists(
        &destination
            .path()
            .join("archive")
            .join("2026-06")
            .join("T-20260629-001.json"),
    );
    assert_file_exists(&source.path().join("active").join("T-20260630-001.json"));
    assert_file_exists(
        &source
            .path()
            .join("archive")
            .join("2026-06")
            .join("T-20260629-001.json"),
    );
    let index: TicketIndex = read_json(&destination.path().join("index.json"))?;
    let active_id = TicketId::parse("T-20260630-001")?;
    let done_id = TicketId::parse("T-20260630-002")?;
    assert!(index.tickets.contains_key(&active_id));
    assert!(!index.tickets.contains_key(&done_id));
    assert_events_contain_imported(destination.path(), "T-20260630-001")?;
    assert_events_contain_imported(destination.path(), "T-20260630-002")?;
    assert_events_contain_imported(destination.path(), "T-20260629-001")?;
    Ok(())
}

#[test]
fn import_store_rejects_duplicate_source_ids() -> Result<(), Box<dyn std::error::Error>> {
    // Given: the same ticket ID appears in source active and archive.
    let source = tempdir()?;
    let destination = tempdir()?;
    write_ticket_fixture(
        &source.path().join("active").join("T-20260630-003.json"),
        json!({"id": "T-20260630-003", "title": "active", "status": "open"}),
    )?;
    write_ticket_fixture(
        &source
            .path()
            .join("archive")
            .join("2026-06")
            .join("T-20260630-003.json"),
        json!({"id": "T-20260630-003", "title": "archived", "status": "done"}),
    )?;
    let store = TicketStore::new(StorePaths::new(destination.path().to_path_buf()));

    // When: the source store is imported.
    let result = store.import_store(source.path());

    // Then: duplicate source IDs are rejected.
    let error = result.err().ok_or("expected duplicate source")?;
    assert!(
        error
            .to_string()
            .contains("duplicate source ticket T-20260630-003")
    );
    Ok(())
}

#[test]
fn import_store_rejects_destination_collision_without_overwrite()
-> Result<(), Box<dyn std::error::Error>> {
    // Given: the destination already has a ticket with a source ID.
    let source = tempdir()?;
    let destination = tempdir()?;
    let existing = json!({
        "id": "T-20260630-004",
        "title": "destination keeps this",
        "status": "open"
    });
    write_ticket_fixture(
        &destination
            .path()
            .join("active")
            .join("T-20260630-004.json"),
        existing.clone(),
    )?;
    write_ticket_fixture(
        &source.path().join("active").join("T-20260630-004.json"),
        json!({"id": "T-20260630-004", "title": "source must not overwrite", "status": "open"}),
    )?;
    let store = TicketStore::new(StorePaths::new(destination.path().to_path_buf()));

    // When: the source store is imported.
    let result = store.import_store(source.path());

    // Then: the collision is rejected and the destination ticket is unchanged.
    let error = result.err().ok_or("expected destination collision")?;
    assert!(
        error
            .to_string()
            .contains("duplicate destination ticket T-20260630-004")
    );
    let unchanged: Value = read_json(
        &destination
            .path()
            .join("active")
            .join("T-20260630-004.json"),
    )?;
    assert_eq!(unchanged["title"], existing["title"]);
    Ok(())
}

fn write_ticket_fixture(path: &Path, mut value: Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    value["priority"] = json!("medium");
    value["type"] = json!("chore");
    value["created"] = json!("2026-06-30T00:00:00Z");
    if value.get("updated").is_none() {
        value["updated"] = json!("2026-06-30T00:00:00Z");
    }
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "expected {} to exist", path.display());
}

fn assert_events_contain_imported(root: &Path, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let events = fs::read_to_string(root.join("events").join(format!("{id}.ndjson")))?;
    assert!(events.contains("\"type\":\"ticket.imported\""));
    Ok(())
}

fn read_json<T>(path: &Path) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}
