//! Error types for the content validation pipeline.
//!
//! `ContentError` represents a single validation failure produced by one of
//! the seven validators in [`crate::validate`]. The pipeline collects these
//! into a `Vec<ContentError>` per validator; an empty vector means "no errors".

/// A single validation error discovered during the content bake pipeline.
#[derive(Debug, Clone)]
pub struct ContentError {
    /// Human-readable description of what failed.
    pub message: String,
    /// Content file that triggered this error, if applicable.
    pub file: Option<String>,
    /// Specific field or identifier within the file, if applicable.
    pub field: Option<String>,
}

impl ContentError {
    /// Create a `ContentError` with only a message.
    pub fn new(message: impl Into<String>) -> Self {
        ContentError {
            message: message.into(),
            file: None,
            field: None,
        }
    }

    /// Create a `ContentError` with a message and source file.
    pub fn in_file(message: impl Into<String>, file: impl Into<String>) -> Self {
        ContentError {
            message: message.into(),
            file: Some(file.into()),
            field: None,
        }
    }

    /// Create a `ContentError` with a message, source file, and specific field.
    pub fn in_field(
        message: impl Into<String>,
        file: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        ContentError {
            message: message.into(),
            file: Some(file.into()),
            field: Some(field.into()),
        }
    }
}

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref file) = self.file {
            write!(f, " [file: {}]", file)?;
        }
        if let Some(ref field) = self.field {
            write!(f, " [field: {}]", field)?;
        }
        Ok(())
    }
}

/// An error that can occur during pack save/load operations.
#[derive(Debug)]
pub enum SaveError {
    /// I/O error (reading or writing files).
    Io(std::io::Error),
    /// The digest file could not be read or parsed.
    Digest(String),
    /// The pack data failed to deserialize.
    Deserialize(String),
    /// The computed digest does not match the stored digest.
    DigestMismatch {
        /// Expected (stored) hex digest.
        expected: String,
        /// Actual (computed) hex digest.
        actual: String,
    },
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "I/O error: {e}"),
            SaveError::Digest(msg) => write!(f, "digest error: {msg}"),
            SaveError::Deserialize(msg) => write!(f, "deserialize error: {msg}"),
            SaveError::DigestMismatch { expected, actual } => {
                write!(f, "digest mismatch: expected {expected}, computed {actual}")
            }
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}
