use std::fmt;

/// First field-level violation found while validating a closed contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// JSON-style path or top-level field name.
    pub field: String,
    /// Stable human-readable reason.
    pub message: String,
}

impl ValidationError {
    pub(crate) fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Fail-closed validation implemented by every public wire contract.
pub trait Validate {
    /// Checks schema identity, bounded fields, and safety invariants.
    ///
    /// # Errors
    ///
    /// Returns the first field-level contract violation.
    fn validate(&self) -> Result<(), ValidationError>;
}
