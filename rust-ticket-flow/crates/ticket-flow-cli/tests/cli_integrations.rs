use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::tempdir;

#[test]
fn cli_import_prints_summary() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a source ticket store with one active ticket.
    let source = tempdir()?;
    let destination = tempdir()?;
    write_ticket_fixture(
        &source.path().join("active").join("T-20260708-004.json"),
        json!({
            "id": "T-20260708-004",
            "title": "CLI import active",
            "status": "open"
        }),
    )?;

    // When/Then: the CLI imports it and prints the summary.
    Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", destination.path())
        .args(["import", &source.path().display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "imported 1 ticket(s): active=1 archived=0",
        ));
    Ok(())
}

#[test]
fn mcp_stdio_lists_calls_tools_and_reports_bad_input() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a source store and MCP JSON-RPC messages over stdio.
    let source = tempdir()?;
    let destination = tempdir()?;
    write_ticket_fixture(
        &source.path().join("active").join("T-20260708-009.json"),
        json!({
            "id": "T-20260708-009",
            "title": "MCP imported",
            "status": "open"
        }),
    )?;
    let stdin = mcp_stdin(&[
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.0.0"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ticket_create","arguments":{"title":"MCP created","type":"feature","priority":"high"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ticket_import","arguments":{"sourceRoot":source.path().display().to_string()}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"ticket_get","arguments":{"id":"T-20260708-009"}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"ticket_update_status","arguments":{"id":"T-20260708-009","status":"doing"}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"ticket_checkpoint","arguments":{"id":"T-20260708-009","phase":"qa","nextType":"agent_action","nextCommand":"cargo test","nextOwner":"codex"}}}),
        json!({"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"ticket_agent_actions","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"ticket_create","arguments":{}}}),
    ]);

    // When: the MCP binary handles the conversation.
    let output = Command::cargo_bin("ticket-flow-mcp")?
        .env("TICKET_FLOW_HOME", destination.path())
        .write_stdin(stdin)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: tools are listed, calls mutate the ticket store, and bad input is an error.
    let responses = json_lines(&output)?;
    let tools = &response_by_id(&responses, 2)?["result"]["tools"];
    assert!(tools.to_string().contains("ticket_create"));
    assert!(tools.to_string().contains("ticket_import"));
    assert!(tools.to_string().contains("ticket_agent_actions"));
    assert!(mcp_text(&response_by_id(&responses, 3)?)?.contains("MCP created"));
    assert_eq!(
        serde_json::from_str::<Value>(&mcp_text(&response_by_id(&responses, 4)?)?)?,
        json!({"imported":1,"active":1,"archived":0})
    );
    assert!(mcp_text(&response_by_id(&responses, 5)?)?.contains("MCP imported"));
    assert!(mcp_text(&response_by_id(&responses, 8)?)?.contains("cargo test"));
    assert_eq!(
        response_by_id(&responses, 9)?["error"]["code"],
        json!(-32602)
    );
    Ok(())
}

#[test]
fn cli_clawhip_prints_sends_and_auto_emit_is_best_effort() -> Result<(), Box<dyn std::error::Error>>
{
    // Given: a ticket store and clawhip auto emit pointing at an unavailable daemon.
    let temp = tempdir()?;
    let output = Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", temp.path())
        .env("TICKET_FLOW_CLAWHIP", "1")
        .env("TICKET_FLOW_CLAWHIP_URL", "http://127.0.0.1:9")
        .env("TICKET_FLOW_CLAWHIP_TIMEOUT_MS", "50")
        .args(["create", "--title", "Clawhip routeable"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let id = String::from_utf8(output)?.trim().to_owned();

    // When: the operator prints and sends the routeable event.
    let printed = Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", temp.path())
        .env("TICKET_FLOW_REPO_PATH", "/repo/ticket-flow")
        .args([
            "clawhip",
            "event",
            &id,
            "--kind",
            "ticket.created",
            "--print",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let server = CaptureServer::start()?;
    Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", temp.path())
        .env("TICKET_FLOW_REPO_PATH", "/repo/ticket-flow")
        .args([
            "clawhip",
            "event",
            &id,
            "--kind",
            "ticket.created",
            "--send",
            "--url",
            &server.url,
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "clawhip: sent ticket.created status=202",
        ));

    // Then: printed and captured payloads are compact ticket-flow IncomingEvent JSON.
    let printed_json: Value = serde_json::from_slice(&printed)?;
    assert_eq!(printed_json["type"], "ticket.created");
    assert_eq!(printed_json["payload"]["provider"], "ticket-flow");
    assert_eq!(printed_json["payload"]["ticket_id"], id);
    assert_eq!(printed_json["payload"]["repo_path"], "/repo/ticket-flow");
    let captured: Value = serde_json::from_str(&server.body()?)?;
    assert_eq!(captured["payload"]["ticket_id"], id);
    Ok(())
}

fn write_ticket_fixture(path: &Path, mut value: Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    value["priority"] = json!("medium");
    value["type"] = json!("chore");
    value["created"] = json!("2026-07-08T00:00:00Z");
    value["updated"] = json!("2026-07-08T00:00:00Z");
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

fn mcp_stdin(messages: &[Value]) -> String {
    let mut stdin = String::new();
    for message in messages {
        stdin.push_str(&message.to_string());
        stdin.push('\n');
    }
    stdin
}

fn json_lines(output: &[u8]) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    String::from_utf8(output.to_owned())?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line)
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })
        })
        .collect()
}

fn response_by_id(responses: &[Value], id: i64) -> Result<Value, io::Error> {
    responses
        .iter()
        .find(|response| response["id"] == json!(id))
        .cloned()
        .ok_or_else(|| io::Error::other(format!("missing response id {id}")))
}

fn mcp_text(response: &Value) -> Result<String, io::Error> {
    response["result"]["content"][0]["text"]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| io::Error::other("missing MCP text content"))
}

struct CaptureServer {
    url: String,
    body_rx: Receiver<io::Result<String>>,
}

impl CaptureServer {
    fn start() -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}", listener.local_addr()?);
        let (body_tx, body_rx) = mpsc::channel();
        thread::spawn(move || {
            let body = capture_http_body(listener);
            let _ = body_tx.send(body);
        });
        Ok(Self { url, body_rx })
    }

    fn body(self) -> io::Result<String> {
        self.body_rx
            .recv()
            .map_err(|error| io::Error::other(error.to_string()))?
    }
}

fn capture_http_body(listener: TcpListener) -> io::Result<String> {
    let (mut stream, _) = listener.accept()?;
    let body = read_http_body(&mut stream)?;
    stream.write_all(
        b"HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nContent-Length: 33\r\n\r\n{\"ok\":true,\"event_id\":\"evt-test\"}",
    )?;
    Ok(body)
}

fn read_http_body(stream: &mut impl Read) -> io::Result<String> {
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    loop {
        let count = stream.read(&mut temp)?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..count]);
        if let Some(body_start) = header_end(&buffer) {
            let headers = String::from_utf8(buffer[..body_start].to_vec())
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            let length = content_length(&headers)?;
            let body_len = buffer.len() - body_start;
            if body_len >= length {
                return String::from_utf8(buffer[body_start..body_start + length].to_vec())
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
            }
        }
    }
    Ok(String::new())
}

fn header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn content_length(headers: &str) -> io::Result<usize> {
    headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))
        .or_else(|| {
            headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
        })
        .and_then(|value| value.trim().parse::<usize>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing content length"))
}
