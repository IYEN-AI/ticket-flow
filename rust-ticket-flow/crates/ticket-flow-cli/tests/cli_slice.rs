use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn cli_create_prints_ticket_id_and_writes_store() {
    // Given: an empty ticket-flow store.
    let temp = tempdir().expect("tempdir");

    // When: the CLI creates a ticket.
    let output = Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args([
            "create",
            "--title",
            "CLI create",
            "--goal",
            "prove create",
            "--acceptance",
            "id is printed",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("T-"))
        .get_output()
        .stdout
        .clone();

    // Then: the ticket file exists in the store.
    let id = String::from_utf8(output).expect("utf8").trim().to_owned();
    assert!(
        temp.path()
            .join("active")
            .join(format!("{id}.json"))
            .exists()
    );
}

#[test]
fn cli_ready_check_json_exits_one_when_missing_fields() {
    // Given: a ticket missing ready data.
    let temp = tempdir().expect("tempdir");
    let output = Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args(["create", "--title", "Missing ready data"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let id = String::from_utf8(output).expect("utf8").trim().to_owned();

    // When: ready-check runs as JSON.
    let output = Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args(["ready-check", &id, "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    // Then: missing fields are machine-readable.
    let value: Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(value["passed"], false);
    assert!(
        value["gates"][0]["missing"]
            .as_array()
            .expect("missing")
            .contains(&Value::String("goal".to_owned()))
    );
    assert!(
        value["gates"][0]["missing"]
            .as_array()
            .expect("missing")
            .contains(&Value::String("acceptance".to_owned()))
    );
    assert!(
        value["gates"][0]["missing"]
            .as_array()
            .expect("missing")
            .contains(&Value::String("current.next_action".to_owned()))
    );
}

#[test]
fn cli_empty_checkpoint_exits_one() {
    // Given: an existing ticket.
    let temp = tempdir().expect("tempdir");
    let output = Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args(["create", "--title", "Empty checkpoint"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let id = String::from_utf8(output).expect("utf8").trim().to_owned();

    // When/Then: checkpoint without fields is rejected.
    Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args(["checkpoint", &id])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "checkpoint requires at least one field",
        ));
}

#[test]
fn cli_context_pack_json_contains_audience() {
    // Given: a ticket with next action and evidence.
    let temp = tempdir().expect("tempdir");
    let output = Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args([
            "create",
            "--title",
            "Rust v2 happy",
            "--type",
            "agent_action",
            "--priority",
            "high",
            "--goal",
            "prove rust implementation",
            "--acceptance",
            "agent queue has command",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let id = String::from_utf8(output).expect("utf8").trim().to_owned();
    Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args([
            "checkpoint",
            &id,
            "--phase",
            "implement",
            "--decision",
            "use Rust store",
            "--evidence",
            "cargo test planned",
            "--next",
            "run agent command",
            "--next-type",
            "agent_action",
            "--next-command",
            "cargo test",
            "--next-owner",
            "codex",
        ])
        .assert()
        .success();
    Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args([
            "evidence",
            "attach",
            &id,
            "--type",
            "test_output",
            "--value",
            "cargo test pass",
        ])
        .assert()
        .success();

    // When: context-pack runs as JSON.
    let output = Command::cargo_bin("ticket-flow")
        .expect("binary")
        .env("TICKET_FLOW_HOME", temp.path())
        .args([
            "context-pack",
            &id,
            "--audience",
            "agent_execution",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: the JSON has the audience and next action command.
    let value: Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(value["audience"], "agent_execution");
    assert_eq!(value["next_action"]["command"], "cargo test");
}
