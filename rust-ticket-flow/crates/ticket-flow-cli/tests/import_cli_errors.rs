use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn cli_import_rejects_missing_source_root() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a source path that does not exist.
    let source_parent = tempdir()?;
    let destination = tempdir()?;
    let source = source_parent.path().join("missing-import-source");

    // When/Then: the CLI imports from the missing source path.
    Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", destination.path())
        .args(["import", &source.display().to_string()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "invalid import source {}",
            source.display()
        )));
    Ok(())
}
