//! Tape error types.
//!
//! All errors produced by the mc_tape crate are `TapeError` variants.
//! Errors implement `std::error::Error` via thiserror.

use thiserror::Error;

/// Errors that can occur during tape construction, serialization,
/// deserialization, recording, or replay.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TapeError {
    /// The tape magic bytes did not match `b"MCTAPE01"`.
    #[error("bad magic: expected MCTAPE01")]
    BadMagic,

    /// Tape entries have non-monotonic or duplicate ticks.
    #[error("non-monotonic tick sequence: tick {tick} <= previous {prev}")]
    NonMonotonicTicks {
        /// The offending tick.
        tick: u64,
        /// The immediately preceding tick.
        prev: u64,
    },

    /// Input data was truncated (unexpected end).
    #[error("tape data truncated")]
    Truncated,

    /// A serialized command had a discriminant with no known variant.
    #[error("unknown command discriminant: {0}")]
    UnknownCommandDiscriminant(u16),

    /// The content digest embedded in the tape does not match the
    /// computed digest of the entries and checkpoints.
    #[error("content digest mismatch")]
    ContentDigestMismatch,

    /// A size bound was exceeded (e.g. too many entries).
    #[error("bounds exceeded: {0}")]
    BoundsExceeded(String),

    /// An I/O error occurred (used when tape data is read from a file).
    #[error("I/O error: {0}")]
    Io(String),

    /// Deserialization failed (postcard or serde error).
    #[error("deserialize error: {0}")]
    Deserialize(String),
}
