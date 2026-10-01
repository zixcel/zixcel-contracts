use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::validation::{map_to_json, validate_digest, validate_identifier, validate_text};
use crate::{
    ReceiptStatusV1, Validate, ValidationError, WORKSPACE_RECEIPT_SCHEMA_V2,
    reject_embedded_secrets,
};

const MAX_DOCUMENT_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceArtifactReferenceV1 {
    pub owner_id: String,
    pub artifact_ref: String,
    pub resolution_ref: String,
    pub schema_id: String,
    pub media_type: String,
    pub digest_sha256: String,
    pub size_bytes: u64,
    pub classification: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProjectionReferenceV1 {
    pub projection_id: String,
    pub revision: u64,
    pub artifact: WorkspaceArtifactReferenceV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceActionReceiptV2 {
    pub schema: String,
    pub receipt_id: String,
    pub request_id: String,
    pub request_digest_sha256: String,
    pub workspace_ref: String,
    pub status: ReceiptStatusV1,
    #[serde(default)]
    pub observations: BTreeMap<String, Value>,
    pub projection: Option<WorkspaceProjectionReferenceV1>,
    #[serde(default)]
    pub artifacts: Vec<WorkspaceArtifactReferenceV1>,
}

impl Validate for WorkspaceArtifactReferenceV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier("owner_id", &self.owner_id)?;
        validate_identifier("artifact_ref", &self.artifact_ref)?;
        validate_identifier("resolution_ref", &self.resolution_ref)?;
        validate_contract_schema("schema_id", &self.schema_id)?;
        validate_text("media_type", &self.media_type, 128)?;
        validate_digest("digest_sha256", &self.digest_sha256)?;
        if self.size_bytes > 16_777_216 {
            return Err(ValidationError::new(
                "size_bytes",
                "must be at most 16777216",
            ));
        }
        if !matches!(
            self.classification.as_str(),
            "public" | "internal" | "internal-confidential" | "restricted-sensitive"
        ) {
            return Err(ValidationError::new("classification", "unsupported value"));
        }
        Ok(())
    }
}

impl Validate for WorkspaceActionReceiptV2 {
    fn validate(&self) -> Result<(), ValidationError> {
        crate::validation::validate_schema("schema", &self.schema, WORKSPACE_RECEIPT_SCHEMA_V2)?;
        validate_identifier("receipt_id", &self.receipt_id)?;
        validate_identifier("request_id", &self.request_id)?;
        validate_digest("request_digest_sha256", &self.request_digest_sha256)?;
        validate_identifier("workspace_ref", &self.workspace_ref)?;
        reject_embedded_secrets(
            "observations",
            &Value::Object(map_to_json(&self.observations)),
        )?;
        let successful = matches!(
            self.status,
            ReceiptStatusV1::Succeeded | ReceiptStatusV1::PartiallySucceeded
        );
        if successful != self.projection.is_some() {
            return Err(ValidationError::new(
                "projection",
                "must match receipt outcome",
            ));
        }
        validate_artifacts(self)?;
        Ok(())
    }
}

fn validate_artifacts(receipt: &WorkspaceActionReceiptV2) -> Result<(), ValidationError> {
    if receipt.artifacts.len() > 256 {
        return Err(ValidationError::new(
            "artifacts",
            "must contain at most 256 entries",
        ));
    }
    let mut ids = BTreeSet::new();
    for artifact in &receipt.artifacts {
        artifact.validate()?;
        if !ids.insert((&artifact.owner_id, &artifact.artifact_ref)) {
            return Err(ValidationError::new(
                "artifacts",
                "references must be unique",
            ));
        }
    }
    if let Some(projection) = &receipt.projection {
        validate_identifier("projection_id", &projection.projection_id)?;
        projection.artifact.validate()?;
        if !receipt.artifacts.contains(&projection.artifact) {
            return Err(ValidationError::new(
                "projection",
                "artifact must be in artifacts",
            ));
        }
    }
    Ok(())
}

fn validate_contract_schema(field: &str, value: &str) -> Result<(), ValidationError> {
    let valid = value
        .split_once("://")
        .is_some_and(|(scheme, path)| !scheme.is_empty() && path.contains("/v"));
    if valid && value.len() <= 256 && !value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            "must be a bounded versioned schema URI",
        ))
    }
}

/// Parses and validates one bounded projection receipt document.
///
/// # Errors
///
/// Returns the first closed-contract violation, including oversized input,
/// unknown fields, an invalid projection reference, or embedded source data.
pub fn parse_workspace_receipt_v2_json(
    source: &[u8],
) -> Result<WorkspaceActionReceiptV2, ValidationError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(ValidationError::new(
            "receipt",
            "document exceeds 1048576 bytes",
        ));
    }
    let receipt = serde_json::from_slice(source)
        .map_err(|_| ValidationError::new("receipt", "invalid closed workspace receipt JSON"))?;
    WorkspaceActionReceiptV2::validate(&receipt)?;
    Ok(receipt)
}
