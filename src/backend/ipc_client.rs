use serde_json::Value;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

// Zenith Control Center → zenithd IPC-client.
//
// De GUI communiceert met de daemon via JSON-RPC 2.0 op dezelfde unix-domain
// socket (FR-01). Deze client is optioneel: als de daemon niet draait, geeft de
// GUI netjes aan dat die uit staat, zonder fouten te veroorzaken.

/// Pad van de daemon-socket (zelfde logica als de daemon zelf).
pub fn daemon_socket_path() -> PathBuf {
    let run_dir = std::env::var("XDG_RUNTIME_DIR").ok();
    if let Some(r) = run_dir {
        if !r.is_empty() {
            return PathBuf::from(r).join("zenith.sock");
        }
    }
    PathBuf::from("/tmp/zenith.sock")
}

/// Stuur één JSON-RPC-verzoek en retourneer het ruwe antwoord, of None als de
/// daemon niet bereikbaar is of het verzoek mislukt.
fn rpc_call(payload: &str) -> Option<Value> {
    let path = daemon_socket_path();
    let stream = UnixStream::connect(path).ok()?;
    let mut stream = stream;
    let _ = stream.write_all(payload.as_bytes());
    let _ = stream.shutdown(Shutdown::Write);

    let mut out = String::new();
    let _ = stream.read_to_string(&mut out);
    let trimmed = out.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str::<Value>(trimmed).ok()
}

/// Controleer of de daemon draait en reageert (ping).
pub fn is_daemon_running() -> bool {
    let req = "{\"jsonrpc\":\"2.0\",\"method\":\"ping\",\"id\":0}";
    match rpc_call(req) {
        Some(resp) => resp.get("result").is_some(),
        None => false,
    }
}