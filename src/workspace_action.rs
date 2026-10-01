use serde::{Deserialize, Serialize};

use crate::validation::{validate_identifier, validate_schema, validate_text};
use crate::{Validate, ValidationError, WORKSPACE_ACTION_SCHEMA_V1};

const MAX_DOCUMENT_BYTES: usize = 1_048_576;

/// Canonical slash-separated path relative to the granted workspace root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkspacePathV1(pub String);

impl Validate for WorkspacePathV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        let value = self.0.as_str();
        validate_text("path", value, 1_024)?;
        if value.starts_with('/')
            || value.contains('\\')
            || value.split('/').any(|part| part.is_empty() || part == "..")
            || value.bytes().any(|byte| byte.is_ascii_control())
        {
            return Err(ValidationError::new(
                "path",
                "must be a canonical slash-separated path below the workspace root",
            ));
        }
        Ok(())
    }
}

/// Effect class used by Hatter for preflight and UI disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceEffectV1 {
    Observe,
    Mutate,
}

/// Closed filesystem and source-control vocabulary implemented by Zixcel adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum WorkspaceActionV1 {
    Inspect {
        path: WorkspacePathV1,
    },
    ReadFile {
        path: WorkspacePathV1,
        maximum_bytes: u32,
    },
    ListDirectory {
        path: WorkspacePathV1,
        maximum_entries: u16,
    },
    SearchText {
        path: WorkspacePathV1,
        query: String,
        maximum_matches: u16,
    },
    ApplyArtifact {
        path: WorkspacePathV1,
        artifact_digest_sha256: String,
    },
    Move {
        from: WorkspacePathV1,
        to: WorkspacePathV1,
    },
    Remove {
        path: WorkspacePathV1,
        recursive: bool,
    },
    SourceStatus,
    SourceDiff {
        base_ref: String,
    },
    SourceApplyPatch {
        patch_digest_sha256: String,
        expected_head_sha256: String,
    },
}

/// Request scoped by opaque workspace and grant references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceActionRequestV1 {
    pub schema: String,
    pub request_id: String,
    pub workspace_ref: String,
    pub grant_ref: String,
    pub action: WorkspaceActionV1,
}

impl WorkspaceActionRequestV1 {
    #[must_use]
    pub const fn effect(&self) -> WorkspaceEffectV1 {
        match self.action {
            WorkspaceActionV1::Inspect { .. }
            | WorkspaceActionV1::ReadFile { .. }
            | WorkspaceActionV1::ListDirectory { .. }
            | WorkspaceActionV1::SearchText { .. }
            | WorkspaceActionV1::SourceStatus
            | WorkspaceActionV1::SourceDiff { .. } => WorkspaceEffectV1::Observe,
            _ => WorkspaceEffectV1::Mutate,
        }
    }
}

impl Validate for WorkspaceActionRequestV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_schema("schema", &self.schema, WORKSPACE_ACTION_SCHEMA_V1)?;
        validate_identifier("request_id", &self.request_id)?;
        validate_identifier("workspace_ref", &self.workspace_ref)?;
        validate_identifier("grant_ref", &self.grant_ref)?;
        self.action.validate()
    }
}

/// Parses a bounded, closed workspace action document.
///
/// # Errors
///
/// Rejects oversized documents, unknown fields and any invalid workspace,
/// grant, path, bound or digest.
pub fn parse_workspace_action_json(
    source: &[u8],
) -> Result<WorkspaceActionRequestV1, ValidationError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(ValidationError::new(
            "request",
            "document exceeds 1048576 bytes",
        ));
    }
    let request = serde_json::from_slice(source)
        .map_err(|_| ValidationError::new("request", "invalid closed workspace action JSON"))?;
    WorkspaceActionRequestV1::validate(&request)?;
    Ok(request)
}
