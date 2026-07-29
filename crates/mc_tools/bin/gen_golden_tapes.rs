//! Generate the locked golden tape artifacts.

use std::path::Path;

const TAPES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes");

fn main() {
    mc_tools::golden_tapes::generate_to(Path::new(TAPES_DIR));
}
