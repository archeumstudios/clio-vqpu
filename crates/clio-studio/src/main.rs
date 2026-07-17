//! Local-only Clio Studio server backed by the real SDK and runtime.

#![forbid(unsafe_code)]

use clio_replay::{ReplayPackage, create_package, verify_package};
use clio_resource::ResourceLimits;
use clio_sdk::{build, estimate, execute, inspect, observe, validate};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::ExitCode,
};

const INDEX: &str = include_str!("../web/index.html");
const STYLE: &str = include_str!("../web/style.css");
const OVERRIDES: &str = include_str!("../web/overrides.css");
const APP: &str = include_str!("../web/app.js");

#[derive(Deserialize)]
struct SourceRequest {
    source: String,
}

#[derive(Deserialize)]
struct ReplayRequest {
    package: ReplayPackage,
}

fn main() -> ExitCode {
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:4317".to_owned());
    let listener = match TcpListener::bind(&address) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("failed to bind {address}: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("Clio Studio: http://{address}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = serve(&mut stream) {
                    eprintln!("request failed: {error}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
    ExitCode::SUCCESS
}

fn serve(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0_u8; 2 * 1024 * 1024];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let (head, body) = request.split_once("\r\n\r\n").unwrap_or((&request, ""));
    let first = head.lines().next().unwrap_or_default();
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or("/");
    let response = route(method, path, body);
    stream.write_all(&response)?;
    Ok(())
}

fn route(method: &str, path: &str, body: &str) -> Vec<u8> {
    match (method, path) {
        ("GET", "/") => response(200, "text/html; charset=utf-8", INDEX.as_bytes()),
        ("GET", "/style.css") => response(200, "text/css; charset=utf-8", STYLE.as_bytes()),
        ("GET", "/overrides.css") => response(200, "text/css; charset=utf-8", OVERRIDES.as_bytes()),
        ("GET", "/app.js") => response(200, "text/javascript; charset=utf-8", APP.as_bytes()),
        ("GET", "/api/examples") => json_response(
            200,
            json!({"bell": include_str!("../../../examples/bell-state/main.clio"), "grover": include_str!("../../../examples/grover/2-qubit/main.clio"), "qft": include_str!("../../../examples/qft/3-qubit-roundtrip/main.clio"), "teleportation": include_str!("../../../examples/teleportation/zero/main.clio")}),
        ),
        ("POST", endpoint) => match serde_json::from_str::<SourceRequest>(body) {
            Ok(request) => source_route(endpoint, &request.source),
            Err(_) if endpoint == "/api/replay/verify" => {
                match serde_json::from_str::<ReplayRequest>(body) {
                    Ok(request) => {
                        result_json(verify_package(&request.package, ResourceLimits::default()))
                    }
                    Err(error) => json_response(400, json!({"ok":false,"error":error.to_string()})),
                }
            }
            Err(error) => json_response(400, json!({"ok":false,"error":error.to_string()})),
        },
        _ => json_response(404, json!({"ok":false,"error":"not found"})),
    }
}

fn source_route(endpoint: &str, source: &str) -> Vec<u8> {
    let limits = ResourceLimits::default();
    match endpoint {
        "/api/build" => result_json(build(source)),
        "/api/run" => result_json(execute(source, limits)),
        "/api/inspect" => result_json(inspect(source, limits)),
        "/api/observe" => result_json(observe(source, limits)),
        "/api/estimate" => result_json(estimate(source, limits)),
        "/api/validate" => result_json(validate(source, limits)),
        "/api/replay/create" => result_json(create_package(source, limits)),
        _ => json_response(404, json!({"ok":false,"error":"unknown API endpoint"})),
    }
}

fn result_json<T: serde::Serialize, E: std::fmt::Display>(result: Result<T, E>) -> Vec<u8> {
    match result {
        Ok(value) => json_response(200, json!({"ok":true,"data":value})),
        Err(error) => json_response(422, json!({"ok":false,"error":error.to_string()})),
    }
}

#[allow(clippy::needless_pass_by_value)] // Call sites construct one-shot JSON response values.
fn json_response(status: u16, value: Value) -> Vec<u8> {
    response(
        status,
        "application/json; charset=utf-8",
        &serde_json::to_vec(&value).unwrap_or_default(),
    )
}

fn response(status: u16, content_type: &str, body: &[u8]) -> Vec<u8> {
    let reason = if status < 300 {
        "OK"
    } else if status == 404 {
        "Not Found"
    } else {
        "Unprocessable Content"
    };
    let mut output = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'self'; style-src 'self'; script-src 'self'\r\n\r\n", body.len()).into_bytes();
    output.extend_from_slice(body);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_run_endpoint_executes_bell() {
        let body = serde_json::to_string(&json!({"source":".shots 8\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n"})).expect("json");
        let response = route("POST", "/api/run", &body);
        assert!(String::from_utf8_lossy(&response).contains("measurement_counts"));
    }
}
