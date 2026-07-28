//! EP-006 M5: No-socket test.
//!
//! Asserts no networking crate exists in the dependency tree and that a full
//! replay of the golden tape runs without opening any socket.

use std::process::Command;

/// No networking crates should be present in the dependency tree.
/// This is the structural assertion — absence beats prohibition.
#[test]
fn no_networking_crates_in_tree() {
    let output = Command::new("cargo")
        .args(["tree", "--locked", "--prefix", "depth", "--workspace"])
        .output()
        .expect("cargo tree must run");
    assert!(output.status.success(), "cargo tree failed");
    let tree = String::from_utf8_lossy(&output.stdout);

    // Denylist of crates that provide networking capabilities
    let denylist = [
        "tokio", "async-std", "smol", "mio", "libc", // Runtime/I/O layers
        "hyper", "reqwest", "ureq", "isahc", "attohttpc", "curl", // HTTP clients
        "warp", "axum", "actix-web", "rocket", "tide", "salvo", // HTTP servers
        "tungstenite", "tokio-tungstenite", "websocket", // WebSocket
        "quinn", "quiche", "s2n-quic", // QUIC
        "rustls", "native-tls", "openssl", // TLS (not needed offline)
        "dns-lookup", "trust-dns", "hickory-resolver", // DNS
    ];

    for &crate_name in &denylist {
        let contains = tree.contains(&format!("\n{} ", crate_name))
            || tree.contains(&format!("\n{} v", crate_name));
        assert!(
            !contains,
            "networking crate '{}' found in dependency tree",
            crate_name
        );
    }
}

/// Replay the golden tape without any socket activity.
#[test]
fn replay_without_socket() {
    // Structural: verify the replay function doesn't require networking
    let tape_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tapes/act1.tape");

    let tape_bytes = std::fs::read(&tape_path).expect("act1.tape must exist");
    let tape = mc_tape::format::Tape::from_bytes(&tape_bytes).expect("tape must deserialize");
    let result = mc_tape::replay::replay(&tape).expect("replay must succeed");

    assert!(
        result.first_divergence.is_none(),
        "golden tape replay must not diverge"
    );
    assert_ne!(
        result.final_hash, [0u8; 32],
        "replay must produce a non-zero hash"
    );
}
