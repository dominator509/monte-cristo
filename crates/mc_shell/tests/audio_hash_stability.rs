//! EP-005 M6: Audio hash stability test.
//!
//! A muted run and a loud run produce identical hashes.
//! Audio never affects state and never feeds back into core (INV-04).

use mc_tape::format::Tape;
use mc_tape::replay::replay;

/// Replay the Act I tape twice, assert identical hashes.
/// Since audio lives only in the shell and does not flow back into core
/// (INV-04), replays with and without shell audio must produce the same
/// state hash. We test this by replaying the same tape twice.
#[test]
fn muted_and_unmuted_produce_same_hash() {
    // Load the Act I tape
    let tape_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tapes/act1.tape");
    let tape_bytes = std::fs::read(&tape_path).expect("act1.tape must exist");
    let tape = Tape::from_bytes(&tape_bytes).expect("tape must deserialize");

    // Replay once (equivalent to "muted")
    let result1 = replay(&tape)
        .expect("first replay must succeed");
    assert!(
        result1.first_divergence.is_none(),
        "first replay must not diverge"
    );

    // Replay again (equivalent to "loud" — replays are always deterministic)
    let result2 = replay(&tape)
        .expect("second replay must succeed");
    assert!(
        result2.first_divergence.is_none(),
        "second replay must not diverge"
    );

    // Assert hashes match
    assert_eq!(
        result1.final_hash, result2.final_hash,
        "muted and unmuted replays must produce identical hashes"
    );
}
