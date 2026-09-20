mod zenith_ipc;

/// zenithd — ZenithOS backend-daemon.
///
/// Start de JSON-RPC 2.0 IPC-server (zie FR-01 van de specificatie op de Unix-domain
/// socket) en blijft draaien. Gebruik:
///
///     ./zenithd              # normaal
///     ./zenithd --debug      # extra logging naar stderr
fn main() {
    let mut debug = false;
    for arg in std::env::args() {
        if arg == "--debug" {
            debug = true;
        }
    }
    match zenith_ipc::serve(debug) {
        Ok(()) => {
            if debug {
                eprintln!("[zenithd] gestopt.");
            }
        }
        Err(e) => {
            eprintln!("[zenithd] fout: {}", e);
            std::process::exit(1);
        }
    }
}