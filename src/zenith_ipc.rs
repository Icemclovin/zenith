use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{create_dir_all, read_to_string};
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
        "sync.get_config" => sync_get_config(params),
        "sync.set_config" => sync_set_config(params),
        "sync.list" => (Some(sync_list()), None),
        "update_module_position" => update_module_position(params),
        "echo" => match params {
            Some(v) => (Some(v), None),
            None => (Some(Value::Null), None),
        },
        _ => (None, Some((-32_601, "Method not found".to_string()))),
    }
}

// ===========================================================================
// Two-way sync (Sprint 1/basis) — FR-01 + FR-03.
//
// `std` biedt hier geen inotify-ondersteuning, dus de daemon werkt op
// "watch-free" two-way sync: elk `sync.get_config`-verzoek leest het
// configbestand live van schijf en elk `sync.set_config`-verzoek schrijft het
// terug. Zo blijft het bestand altijd de Single Source of Truth en blijven
// handmatige (externe) aanpassingen zichtbaar in de GUI — precies de intentie
// van FR-01/FR-03. Wanneer inotify beschikbaar komt, kan hier een watcher bovenop.
//
// Veiligheid: alleen een vaste allow-list van door Zenith beheerde bestanden is
// via de daemon lees-/schrijfbaar (geen willekeurig pad).

/// Vertaalt een logische naam naar het absolute pad van een beheerd bestand.
fn known_config_path(key: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    match key {
        "zenith-hypr" => Some(PathBuf::from(home).join(".config/hypr/zenith.conf")),
        "quickshell" => Some(PathBuf::from(home).join(".config/quickshell/zenith-shell.json")),
        "waybar" => Some(PathBuf::from(home).join(".config/waybar/config")),
        _ => None,
    }
}

/// Haal een stringveld op uit de params (als owned String).
fn param_str(params: &Option<Value>, key: &str) -> String {
    match params {
        Some(v) => match v.get(key) {
            Some(x) => match x.as_str() {
                Some(s) => s.to_string(),
                None => String::new(),
            },
            None => String::new(),
        },
        None => String::new(),
    }
}

/// Bouw een JSON-object met één veld.
fn single_object(key: &str, value: Value) -> Value {
    let mut obj = serde_json::from_str::<Value>("{}").unwrap_or_default();
    if let Some(m) = obj.as_object_mut() {
        m.insert(key.to_string(), value);
    }
    obj
}

/// sync.get_config — lees een beheerd configbestand en retourneer de inhoud.
fn sync_get_config(params: Option<Value>) -> (Option<Value>, Option<(i32, String)>) {
    let key = param_str(&params, "path");
    match known_config_path(&key) {
        Some(path) if path.is_file() => match read_to_string(&path) {
            Ok(content) => (Some(single_object("content", Value::String(content))), None),
            Err(e) => (None, Some((-1, format!("Lezen mislukt: {}", e)))),
        },
        _ => (None, Some((-1, "Onbekend of ontbrekend configbestand".to_string()))),
    }
}

/// sync.set_config — schrijf de opgegeven inhoud terug naar een beheerd bestand.
fn sync_set_config(params: Option<Value>) -> (Option<Value>, Option<(i32, String)>) {
    let key = param_str(&params, "path");
    let content = param_str(&params, "content");
    let path = match known_config_path(&key) {
        Some(p) => p,
        None => return (None, Some((-1, "Onbekend configbestand".to_string()))),
    };
    match write_text_atomic(&path, &content) {
        Ok(()) => (Some(single_object("written", Value::Bool(true))), None),
        Err(e) => (None, Some((-1, format!("Schrijven mislukt: {}", e)))),
    }
}

/// Schrijft tekst atomair weg: eerst naar een tijdelijke file, dan `rename`.
/// Zo ziet Quickshell nooit een half weggeschreven configuratie (voorkomt crash
/// bij herladen — Taak 2).
fn write_text_atomic(path: &PathBuf, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

// ===========================================================================
// Drag-and-Drop Canvas — update_module_position (H5).
//
// Verplaats een statusbalkmodule naar een andere uitlijning (left/center/right)
// en een nieuwe index, schrijf de quickshell-config atomair weg en stuur
// vervolgens SIGUSR1 naar Quickshell voor een live redraw.

/// Pad naar het door Zenith beheerde quickshell-schelsysteembestand.
fn quickshell_json_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    Some(PathBuf::from(home).join(".config/quickshell/zenith-shell.json"))
}

/// Haal een geheel getal uit de params.
fn param_i64(params: &Option<Value>, key: &str) -> Option<i64> {
    match params {
        Some(v) => match v.get(key) {
            Some(x) => x.as_i64(),
            None => None,
        },
        None => None,
    }
}

/// update_module_position — verplaats een module naar slot+index, atomair wegschrijven,
/// en laat Quickshell live hertekenen (SIGUSR1).
fn update_module_position(params: Option<Value>) -> (Option<Value>, Option<(i32, String)>) {
    let module_id = param_str(&params, "module_id");
    let alignment = param_str(&params, "alignment");
    let new_index = param_i64(&params, "new_index").unwrap_or(0);
    if module_id.is_empty() {
        return (None, Some((-1, "module_id ontbreekt".to_string())));
    }
    if !matches!(alignment.as_str(), "left" | "center" | "right") {
        return (None, Some((-1, "alignment moet left/center/right zijn".to_string())));
    }
    let path = match quickshell_json_path() {
        Some(p) => p,
        None => return (None, Some((-1, "Geen home-map".to_string()))),
    };
    let content = match read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return (None, Some((-1, "zenith-shell.json niet gevonden".to_string()))),
    };
    let mut root = match serde_json::from_str::<Value>(&content) {
        Ok(v) => v,
        Err(e) => return (None, Some((-1, format!("JSON parsefout: {}", e)))),
    };

    // Navigeer naar data.modules.
    let moved = {
        let modules = root.get_mut("modules");
        let modules = match modules {
            Some(m) if m.is_object() => m.as_object_mut().unwrap(),
            _ => return (None, Some((-1, "modules-veld ontbreekt".to_string()))),
        };

        // Haal uit elke slot de module weg.
        for slot in ["left", "center", "right"] {
            if let Some(arr) = modules.get_mut(slot) {
                if let Some(a) = arr.as_array_mut() {
                    a.retain(|x| !(x.as_str().unwrap_or("") == module_id));
                }
            }
        }

        // Plaats op de nieuwe index in het doel-slot.
        let target = modules.get_mut(alignment.as_str());
        match target {
            Some(t) if t.is_array() => {
                if let Some(a) = t.as_array_mut() {
                    let idx = new_index.clamp(0, a.len() as i64) as usize;
                    a.insert(idx, Value::String(module_id.clone()));
                    true
                } else {
                    false
                }
            }
            Some(t) if t.is_null() => {
                // Nog geen array aangelegd.
                modules.insert(
                    alignment.clone(),
                    Value::Array(vec![Value::String(module_id.clone())]),
                );
                true
            }
            _ => false,
        }
    };

    if !moved {
        return (None, Some((-1, "Doel-slot is geen array".to_string())));
    }

    match serde_json::to_string_pretty(&root) {
        Ok(json) => match write_text_atomic(&path, &json) {
            Ok(()) => {
                let reload = quickshell_reload();
                let _ = reload;
                (Some(single_object("ok", Value::Bool(true))), None)
            }
            Err(e) => (None, Some((-1, format!("Wegschrijven mislukt: {}", e)))),
        },
        Err(e) => (None, Some((-1, format!("JSON serialisatiefout: {}", e)))),
    }
}

/// sync.list — lijst de beheerde configbestanden.
fn sync_list() -> Value {
    let mut arr = serde_json::from_str::<Value>("[]").unwrap_or_default();
    if let Some(a) = arr.as_array_mut() {
        for k in ["zenith-hypr", "quickshell", "waybar"] {
            a.push(single_object("path", Value::String(k.to_string())));
        }
    }
    arr
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

#[test]
fn test_known_config_path_allow_list() {
    // Bekende namen worden naar de juiste plek vertaald.
    assert!(known_config_path("zenith-hypr").is_some());
    assert!(known_config_path("quickshell").is_some());
    assert!(known_config_path("waybar").is_some());
    // Onbekende namen mogen NOOIT naar een pad vertaald worden (geen willekeurig schrijven).
    assert!(known_config_path("../../etc/passwd").is_none());
    assert!(known_config_path("/etc/shadow").is_none());
    assert!(known_config_path("").is_none());
    assert!(known_config_path("..").is_none());
}

#[test]
fn test_set_config_unknown_returns_error() {
    let params = serde_json::from_str::<Value>(
        r#"{"path":"/etc/shadow","content":"x"}"#
    ).unwrap_or_default();
    let (res, err) = sync_set_config(Some(params));
    assert!(res.is_none());
    assert!(err.is_some());
}

#[test]
fn test_param_str_extracts_fields() {
    let params_a = serde_json::from_str::<Value>(
        r#"{"path":"waybar","content":"hello"}"#
    ).unwrap_or_default();
    let params_b = params_a.clone();
    assert_eq!(param_str(&Some(params_a), "path"), "waybar");
    assert_eq!(param_str(&Some(params_b), "content"), "hello");
    // Ontbrekend veld geeft lege string.
    let empty = serde_json::from_str::<Value>("{}").unwrap_or_default();
    assert_eq!(param_str(&Some(empty), "nope"), "");
}

/// update_module_position met een tijdelijke HOME schrijven we naar
/// ~/.config/quickshell/zenith-shell.json zodat de test losstaat van de echte config.
#[test]
fn test_update_module_position_moves_module() {
    let fake_home = std::path::PathBuf::from("/tmp/zenith-ipc-test-home");
    std::env::set_var("HOME", &fake_home);

    let qs_dir = fake_home.join(".config/quickshell");
    let _ = std::fs::create_dir_all(&qs_dir);
    let shell_path = qs_dir.join("zenith-shell.json");
    let _ = std::fs::write(
        &shell_path,
        r#"{ "modules": { "left": ["launcher","workspaces"], "center": [], "right": ["clock"] } }"#,
    );

    // Verplaats "clock" naar center-index 0.
    let params = serde_json::from_str::<Value>(
        r#"{"module_id":"clock","new_index":0,"alignment":"center"}"#,
    ).unwrap_or_default();
    let (res, err) = update_module_position(Some(params));
    assert!(err.is_none(), "geen fout verwacht: {:?}", err);
    assert!(res.is_some());

    // Verifieer de nieuwe structuur op schijf.
    let now = std::fs::read_to_string(&shell_path).unwrap_or_default();
    let root: Value = serde_json::from_str(&now).unwrap_or_default();
    let right = root["modules"]["right"].as_array();
    assert!(right.is_none() || right.map(|a| a.is_empty()).unwrap_or(true),
        "clock mag niet meer in right staan");
    let center = root["modules"]["center"].as_array().unwrap().clone();
    assert_eq!(center, vec![Value::String("clock".to_string())]);

    std::env::remove_var("HOME");
    let _ = std::fs::remove_dir_all(fake_home);
}

/// update_module_position weigert een ongeldige alignment.
#[test]
fn test_update_module_position_rejects_bad_alignment() {
    let params = serde_json::from_str::<Value>(
        r#"{"module_id":"clock","new_index":0,"alignment":"up"}"#,
    ).unwrap_or_default();
    let (res, err) = update_module_position(Some(params));
    assert!(res.is_none());
    assert!(err.is_some());
}
