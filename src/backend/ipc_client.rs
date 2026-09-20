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

/// Stuur een JSON-RPC-verzoek met params-object en retourneer de `result` van de
/// daemon, of None als die onbereikbaar is of een fout teruggeeft.
fn rpc_call_with_params(method: &str, params: Value) -> Option<Value> {
    let mut req = serde_json::from_str::<Value>("{}").unwrap_or_default();
    if let Some(o) = req.as_object_mut() {
        o.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
        o.insert("method".to_string(), Value::String(method.to_string()));
        o.insert("id".to_string(), Value::Number(1i64.into()));
        o.insert("params".to_string(), params);
    }
    let payload = serde_json::to_string(&req).unwrap_or_default();
    match rpc_call(&payload) {
        Some(resp) => resp.get("result").cloned(),
        None => None,
    }
}

/// Verplaats een statusbalkmodule naar een andere uitlijning en index via de
/// daemon (`update_module_position`), gevolgd door een SIGUSR1-redraw.
/// Returnt `true` als de daemon de wijziging accepteerde.
pub fn update_module_position(module_id: &str, new_index: i64, alignment: &str) -> bool {
    let mut params = serde_json::from_str::<Value>("{}").unwrap_or_default();
    if let Some(o) = params.as_object_mut() {
        o.insert("module_id".to_string(), Value::String(module_id.to_string()));
        o.insert("new_index".to_string(), Value::Number(new_index.into()));
        o.insert("alignment".to_string(), Value::String(alignment.to_string()));
    }
    match rpc_call_with_params("update_module_position", params) {
        Some(result) => result.get("ok").and_then(|b| b.as_bool()).unwrap_or(false),
        None => false,
    }
}

/// Lees een beheerd configbestand via de daemon en retourneer de inhoud.
/// (Bestanddeel van de openbare IPC-client API voor toekomstige callers.)
#[allow(dead_code)]
pub fn get_managed_config(key: &str) -> Option<String> {
    let mut params = serde_json::from_str::<Value>("{}").unwrap_or_default();
    if let Some(o) = params.as_object_mut() {
        o.insert("path".to_string(), Value::String(key.to_string()));
    }
    match rpc_call_with_params("sync.get_config", params) {
        Some(result) => result.get("content").and_then(|c| c.as_str()).map(|s| s.to_string()),
        None => None,
    }
}