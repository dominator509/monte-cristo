#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the tape parser: feed arbitrary bytes and assert it doesn't crash.
    let _ = mc_tape::format::Tape::from_bytes(data);
});
