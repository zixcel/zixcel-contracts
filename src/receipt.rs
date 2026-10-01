use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::validation::{map_to_json, validate_identifier, validate_schema, validate_text};
use crate::{RECEIPT_SCHEMA_V1, Validate, ValidationError, reject_embedded_secrets};

/// Outcome reported by the separately authorized executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptStatusV1 {
    Succeeded,
    PartiallySucceeded,
    Failed,
    Rejected,
}

/// Digest-only artifact reference; artifact content is not copied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptArtifactV1 {
    pub artifact_id: String,
    pub media_type: String,
    pub digest_sha256: String,
}

/// Closed execution result returned to a workflow engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorReceiptV1 {
    pub schema: String,
    pub receipt_id: String,
    pub request_id: String,
    pub plan_id: String,
    pub provider: String,
    pub status: ReceiptStatusV1,
    #[serde(default)]
    pub observations: BTreeMap<String, Value>,
    #[serde(default)]
    pub artifacts: Vec<ReceiptArtifactV1>,
}

impl Validate for ConnectorReceiptV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_schema("schema", &self.schema, RECEIPT_SCHEMA_V1)?;
        validate_identifier("receipt_id", &self.receipt_id)?;
        validate_identifier("request_id", &self.request_id)?;
        validate_identifier("plan_id", &self.plan_id)?;
        validate_identifier("provider", &self.provider)?;
        reject_embedded_secrets(
            "observations",
            &Value::Object(map_to_json(&self.observations)),
        )?;
        if self.artifacts.len() > 256 {
            return Err(ValidationError::new(
                "artifacts",
                "must contain at most 256 entries",
            ));
        }
        let mut identifiers = BTreeSet::new();
        for (index, artifact) in self.artifacts.iter().enumerate() {
            validate_identifier(
                &format!("artifacts[{index}].artifact_id"),
                &artifact.artifact_id,
            )?;
            if !identifiers.insert(&artifact.artifact_id) {
                return Err(ValidationError::new(
                    format!("artifacts[{index}].artifact_id"),
                    "artifact identifiers must be unique",
                ));
            }
            validate_text(
                &format!("artifacts[{index}].media_type"),
                &artifact.media_type,
                128,
            )?;
            if artifact.digest_sha256.len() != 64
                || !artifact
                    .digest_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(ValidationError::new(
                    format!("artifacts[{index}].digest_sha256"),
                    "must be a 64-character hexadecimal SHA-256 digest",
                ));
            }
        }
        Ok(())
    }
}
