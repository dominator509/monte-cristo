#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the pack parser: feed arbitrary bytes through postcard deserialization.
    let _ = mc_data::pack::Pack::load_from_bytes(data);
});
