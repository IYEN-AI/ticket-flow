use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde_json::{Value, json};

mod tools;

pub(crate) fn run_stdio() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = handle_line(&line) {
            serde_json::to_writer(&mut stdout, &response)?;
            stdout.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn handle_line(line: &str) -> Option<Value> {
    match serde_json::from_str::<Value>(line) {
        Ok(request) => handle_request(request),
        Err(error) => Some(error_response(Value::Null, -32700, &error.to_string())),
    }
}

fn handle_request(request: Value) -> Option<Value> {
    let id = request.get("id").cloned();
    let method = request.get("method").and_then(Value::as_str);
    match method {
        Some("initialize") => id.map(initialize_response),
        Some("notifications/initialized") => None,
        Some("tools/list") => id.map(tools_list_response),
        Some("tools/call") => Some(call_tool_response(id.unwrap_or(Value::Null), request)),
        Some(other) => {
            id.map(|value| error_response(value, -32601, &format!("unknown method {other}")))
        }
        None => Some(error_response(
            id.unwrap_or(Value::Null),
            -32600,
            "missing method",
        )),
    }
}

fn initialize_response(id: Value) -> Value {
    success_response(
        id,
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "ticket-flow", "version": env!("CARGO_PKG_VERSION")}
        }),
    )
}

fn tools_list_response(id: Value) -> Value {
    success_response(id, json!({ "tools": tools::tool_definitions() }))
}

fn call_tool_response(id: Value, request: Value) -> Value {
    let Some(params) = request.get("params") else {
        return error_response(id, -32602, "tools/call requires params");
    };
    match tools::call(params) {
        Ok(result) => success_response(id, result),
        Err(error) => error_response(id, -32602, &error.to_string()),
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}
