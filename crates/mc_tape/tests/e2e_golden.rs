//! EP-007 M3: Golden tape e2e test.
//!
//! Verifies the golden-full.tape and golden-smoke.tape files exist on disk,
//! deserialize correctly, and replay with matching hashes.

use mc_tape::format::Tape;

const TAPES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes");

#[test]
fn e2e_golden_full() {
    let path = format!("{}/golden-full.tape", TAPES_DIR);
    let data = std::fs::read(&path).expect("golden-full.tape should exist");
    let tape = Tape::from_bytes(&data).expect("golden-full.tape deserialization");
    assert!(!tape.is_empty(), "golden-full tape should have entries");
    assert_eq!(tape.magic, *b"MCTAPE01");

    let result = mc_tape::replay::replay(&tape).expect("golden-full replay");
    assert!(
        result.first_divergence.is_none(),
        "golden-full divergence: {:?}",
        result.first_divergence
    );
    assert_eq!(result.final_hash, tape.final_hash, "golden-full hash match");
}

#[test]
fn e2e_golden_smoke() {
    let path = format!("{}/golden-smoke.tape", TAPES_DIR);
    let data = std::fs::read(&path).expect("golden-smoke.tape should exist");
    let tape = Tape::from_bytes(&data).expect("golden-smoke.tape deserialization");
    assert!(!tape.is_empty(), "golden-smoke tape should have entries");
    assert_eq!(tape.magic, *b"MCTAPE01");

    let result = mc_tape::replay::replay(&tape).expect("golden-smoke replay");
    assert!(
        result.first_divergence.is_none(),
        "golden-smoke divergence: {:?}",
        result.first_divergence
    );
    assert_eq!(
        result.final_hash, tape.final_hash,
        "golden-smoke hash match"
    );
}

#[test]
fn e2e_hashes_file_exists() {
    let path = format!("{}/HASHES.txt", TAPES_DIR);
    let content = std::fs::read_to_string(&path).expect("HASHES.txt should exist");
    assert!(
        content.contains("golden-full.tape"),
        "HASHES.txt should contain golden-full.tape hash"
    );
    assert!(
        content.contains("golden-smoke.tape"),
        "HASHES.txt should contain golden-smoke.tape hash"
    );
}
