use crate::validation::{validate_digest, validate_text};
use crate::{Validate, ValidationError, WorkspaceActionV1};

impl Validate for WorkspaceActionV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::Inspect { path }
            | Self::ReadFile { path, .. }
            | Self::ListDirectory { path, .. }
            | Self::ApplyArtifact { path, .. }
            | Self::Remove { path, .. } => path.validate(),
            Self::SearchText {
                path,
                query,
                maximum_matches,
            } => {
                path.validate()?;
                validate_text("query", query, 1_024)?;
                validate_count("maximum_matches", *maximum_matches)
            }
            Self::Move { from, to } => {
                from.validate()?;
                to.validate()
            }
            Self::SourceStatus => Ok(()),
            Self::SourceDiff { base_ref } => validate_source_ref(base_ref),
            Self::SourceApplyPatch {
                patch_digest_sha256,
                expected_head_sha256,
            } => {
                validate_digest("patch_digest_sha256", patch_digest_sha256)?;
                validate_digest("expected_head_sha256", expected_head_sha256)
            }
        }?;
        validate_action_bound(self)
    }
}

fn validate_action_bound(action: &WorkspaceActionV1) -> Result<(), ValidationError> {
    match action {
        WorkspaceActionV1::ReadFile { maximum_bytes, .. }
            if *maximum_bytes == 0 || *maximum_bytes > 1_048_576 =>
        {
            Err(ValidationError::new(
                "maximum_bytes",
                "must be within 1..=1048576",
            ))
        }
        WorkspaceActionV1::ListDirectory {
            maximum_entries, ..
        } => validate_count("maximum_entries", *maximum_entries),
        WorkspaceActionV1::ApplyArtifact {
            artifact_digest_sha256,
            ..
        } => validate_digest("artifact_digest_sha256", artifact_digest_sha256),
        _ => Ok(()),
    }
}

fn validate_source_ref(value: &str) -> Result<(), ValidationError> {
    validate_text("base_ref", value, 256)?;
    if value.bytes().any(|byte| byte.is_ascii_control()) {
        Err(ValidationError::new(
            "base_ref",
            "must not contain ASCII control characters",
        ))
    } else {
        Ok(())
    }
}

fn validate_count(field: &str, value: u16) -> Result<(), ValidationError> {
    if value == 0 || value > 4_096 {
        Err(ValidationError::new(field, "must be within 1..=4096"))
    } else {
        Ok(())
    }
}
