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
