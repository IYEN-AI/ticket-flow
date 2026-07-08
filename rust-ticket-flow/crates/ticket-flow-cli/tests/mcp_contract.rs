use std::io;

use assert_cmd::Command;
use serde_json::{Value, json};
use tempfile::tempdir;

#[test]
fn mcp_tools_list_exposes_required_fields_and_enums() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a tools/list request.
    let destination = tempdir()?;
    let stdin = format!(
        "{}\n{}\n",
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.0.0"}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"})
    );

    // When: the MCP server lists tools.
    let output = Command::cargo_bin("ticket-flow-mcp")?
        .env("TICKET_FLOW_HOME", destination.path())
        .write_stdin(stdin)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Then: clients can discover required fields and allowed enums.
    let responses = json_lines(&output)?;
    let tools = response_by_id(&responses, 2)?["result"]["tools"]
        .as_array()
        .ok_or_else(|| io::Error::other("tools must be an array"))?
        .clone();
    let create = tool_by_name(&tools, "ticket_create")?;
    assert_eq!(create["inputSchema"]["required"], json!(["title"]));
    let status = tool_by_name(&tools, "ticket_update_status")?;
    assert_eq!(status["inputSchema"]["required"], json!(["id", "status"]));
    assert_eq!(
        status["inputSchema"]["properties"]["status"]["enum"],
        json!(["open", "doing", "review", "blocked", "done"])
    );
    Ok(())
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

fn tool_by_name(tools: &[Value], name: &str) -> Result<Value, io::Error> {
    tools
        .iter()
        .find(|tool| tool["name"] == json!(name))
        .cloned()
        .ok_or_else(|| io::Error::other(format!("missing tool {name}")))
}
