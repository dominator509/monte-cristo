#![forbid(unsafe_code)]
//! INV-01: pure, deterministic, no I/O. See ARCHITECTURE.md section 5.

pub mod flags;
pub mod fx;
pub mod hash;
pub mod ids;
pub mod rng;
pub mod step;
pub mod world;

/// The number of regions in the campaign. SPEC-009 section 1 is authoritative.
pub const REGION_COUNT: usize = 15;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_count_matches_spec() {
        // INV-06: content is data, but the region count is a structural fact the
        // bake validates against, so it lives in code as the single source.
        assert_eq!(REGION_COUNT, 15);
    }
}
