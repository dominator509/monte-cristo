#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the save parser: feed arbitrary bytes and assert it doesn't crash.
    let _ = mc_data::save::Save::load(data);
});
