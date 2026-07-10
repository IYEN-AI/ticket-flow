use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn clawhip_strict_send_rejects_http_500() -> Result<(), Box<dyn std::error::Error>> {
    // Given: a ticket and a clawhip endpoint returning HTTP 500.
    let temp = tempdir()?;
    let output = Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", temp.path())
        .args(["create", "--title", "Strict clawhip"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let id = String::from_utf8(output)?.trim().to_owned();
    let server = StatusServer::start(500)?;

    // When/Then: strict send exits non-zero and reports the HTTP status.
    Command::cargo_bin("ticket-flow")?
        .env("TICKET_FLOW_HOME", temp.path())
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
        .failure()
        .stderr(predicate::str::contains("clawhip returned HTTP 500"));
    let _ = server.body()?;
    Ok(())
}

struct StatusServer {
    url: String,
    body_rx: Receiver<io::Result<String>>,
}

impl StatusServer {
    fn start(status: u16) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}", listener.local_addr()?);
        let (body_tx, body_rx) = mpsc::channel();
        thread::spawn(move || {
            let body = capture_http_body(listener, status);
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

fn capture_http_body(listener: TcpListener, status: u16) -> io::Result<String> {
    let (mut stream, _) = listener.accept()?;
    let body = read_http_body(&mut stream)?;
    let response = format!(
        "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: 12\r\n\r\n{{\"ok\":false}}"
    );
    stream.write_all(response.as_bytes())?;
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
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing content-length"))
}
