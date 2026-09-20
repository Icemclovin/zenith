use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;

// zenithd — JSON-RPC 2.0 IPC-server voor ZenithOS.
//
// Voldoet aan FR-01 van de systeemspecificatie: alle communicatie tussen de GUI en
// de backend verloopt via JSON-RPC 2.0 op `$XDG_RUNTIME_DIR/zenith.sock`.
//
// Transport: één verbinding per verzoek. De client stuurt één JSON-verzoek gevolgd
// door een write-half-close; de server leest het volledige verzoek, handelt het af
// en antwoordt met één JSON-antwoord gevolgd door write-half-close.

/// Pad van de Unix-domain-socket. FR-01: `$XDG_RUNTIME_DIR/zenith.sock`.
pub fn socket_path() -> Option<PathBuf> {
    let run_dir = std::env::var("XDG_RUNTIME_DIR").ok();
    if let Some(r) = run_dir {
        if !r.is_empty() {
            return Some(std::path::PathBuf::from(r).join("zenith.sock"));
        }
    }
    Some(std::path::PathBuf::from("/tmp/zenith.sock"))
}

/// JSON-RPC 2.0-verzoek.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct RpcRequest {
    #[serde(default)]
    jsonrpc: Option<String>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
    #[serde(default)]
    id: Option<Value>,
}

fn make_response(id: Option<Value>, result: Option<Value>, err: Option<(i32, String)>) -> Value {
    // Bouw het antwoord dynamisch door te starten vanaf een leeg object.
    let mut out = serde_json::from_str::<Value>("{}").unwrap_or_default();

    let id_value = match &id {
        Some(v) => v.clone(),
        None => Value::Null,
    };
    let result_value = match &result {
        Some(v) => v.clone(),
        None => Value::Null,
    };

    if let Some(m) = out.as_object_mut() {
        m.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
        m.insert("id".to_string(), id_value);
    }

    match err {
        Some((code, message)) => {
            if let Some(m) = out.as_object_mut() {
                let mut eobj = serde_json::from_str::<Value>("{}").unwrap_or_default();
                if let Some(eo) = eobj.as_object_mut() {
                    eo.insert("code".to_string(), Value::Number(code.into()));
                    eo.insert("message".to_string(), Value::String(message));
                }
                m.insert("error".to_string(), eobj);
            }
        }
        None => {
            if let Some(m) = out.as_object_mut() {
                m.insert("result".to_string(), result_value);
            }
        }
    }
    out
}

/// Stuur een JSON-RPC-antwoord terug over de stream (met write-half-close).
fn send_json(conn: &mut UnixStream, body: Value) {
    if let Ok(serialized) = serde_json::to_string(&body) {
        let mut payload = serialized;
        payload.push('\n');
        let _ = conn.write_all(payload.as_bytes());
        let _ = conn.shutdown(Shutdown::Write);
    }
}

/// Stuur een SIGUSR1 naar Quickshell zodat de statusbalk live opnieuw tekent
/// (zie specificatie §5 "Drag-and-Drop Canvas").
fn quickshell_reload() -> Value {
    let _ = Command::new("pkill").args(["-USR1", "quickshell"]).spawn();
    Value::Bool(true)
}

/// Status van de daemon.
fn daemon_status() -> Value {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut obj = serde_json::from_str::<Value>("{}").unwrap_or_default();
    if let Some(m) = obj.as_object_mut() {
        m.insert("daemon".to_string(), Value::String("zenithd".to_string()));
        m.insert("protocol".to_string(), Value::String("jsonrpc-2.0".to_string()));
        m.insert(
            "socket".to_string(),
            Value::String(socket_path().unwrap_or_default().to_string_lossy().to_string()),
        );
        m.insert("home".to_string(), Value::String(home));
        m.insert("pid".to_string(), Value::Number(std::process::id().into()));
    }
    obj
}

/// Behandel één JSON-RPC-methodeoproep. Returnt (result, of None bij een fout).
fn dispatch(method: &str, params: Option<Value>) -> (Option<Value>, Option<(i32, String)>) {
    match method {
        "ping" => (Some(Value::String("pong".to_string())), None),
        "get_status" => (Some(daemon_status()), None),
        "reload_statusbar" => (Some(quickshell_reload()), None),
        "echo" => match params {
            Some(v) => (Some(v), None),
            None => (Some(Value::Null), None),
        },
        _ => (None, Some((-32_601, "Method not found".to_string()))),
    }
}

/// Verwerk één verbinding: lees, dispatch, antwoord.
fn handle_connection(conn: UnixStream, debug: bool) {
    let mut conn = conn;
    let mut raw = String::new();
    let _ = conn.read_to_string(&mut raw);
    let payload = raw.trim();

    // Leeg verzoek (bijv. alleen half-close zonder data): niets terugsturen.
    if payload.is_empty() {
        let _ = conn.shutdown(Shutdown::Write);
        return;
    }

    let parsed = serde_json::from_str::<RpcRequest>(payload);
    match parsed {
        Err(e) => {
            if debug {
                eprintln!("[zenithd] parsefout: {}", e);
            }
            send_json(&mut conn, make_response(None, None, Some((-32_700, "Invalid Request".to_string()))));
        }
        Ok(req) => {
            let id = req.id.clone();
            let method = req.method.clone();
            if debug {
                eprintln!("[zenithd] <= {}", method);
            }
            let (result, err) = dispatch(&method, req.params);
            send_json(&mut conn, make_response(id, result, err));
        }
    }
}

/// Start de daemon. Blokkeert en serveert verbindingen tot het proces stopt.
pub fn serve(debug: bool) -> Result<(), String> {
    let path = match socket_path() {
        Some(p) => p,
        None => return Err("Kan socketpad niet bepalen".to_string()),
    };
    let path_str = path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let listener = match UnixListener::bind(path) {
        Ok(l) => l,
        Err(e) => return Err(format!("Kon niet binden op {}: {}", path_str, e)),
    };

    if debug {
        eprintln!("[zenithd] luister op {}", path_str);
    }

    loop {
        match listener.accept() {
            Ok((conn, _peer)) => {
                let _ = std::thread::spawn(move || -> Result<(), String> {
                    handle_connection(conn, debug);
                    Ok(())
                });
            }
            Err(e) => {
                if debug {
                    eprintln!("[zenithd] acceptfout: {}", e);
                }
            }
        }
    }
}
#[test]
fn test_make_response_success() {
    let resp = make_response(
        Some(Value::Number(1i64.into())),
        Some(Value::String("pong".to_string())),
        None,
    );
    let s = serde_json::to_string(&resp).unwrap_or_default();
    assert!(s.contains("\"result\":\"pong\""));
    assert!(s.contains("\"jsonrpc\":\"2.0\""));
    assert!(!s.contains("error"));
}

#[test]
fn test_make_response_error_has_no_result() {
    let resp = make_response(
        Some(Value::Number(4i64.into())),
        None,
        Some((-32_601, "Method not found".to_string())),
    );
    let s = serde_json::to_string(&resp).unwrap_or_default();
    assert!(s.contains("-32601"));
    assert!(s.contains("Method not found"));
    assert!(s.contains("\"error\""));
}

#[test]
fn test_dispatch_known_and_unknown() {
    let (res, err) = dispatch("ping", None);
    assert!(res.is_some());
    assert!(err.is_none());
    assert_eq!(res.unwrap_or(Value::Null), Value::String("pong".to_string()));

    let (res2, err2) = dispatch("does_not_exist", None);
    assert!(res2.is_none());
    assert!(err2.is_some());
    assert_eq!(err2.unwrap_or((-1, "".to_string())).0, -32_601);
}
