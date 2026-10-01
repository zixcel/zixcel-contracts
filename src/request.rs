use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::validation::{
    map_to_json, validate_identifier, validate_schema, validate_secret_ref, validate_secret_refs,
    validate_text,
};
use crate::{REQUEST_SCHEMA_V1, Validate, ValidationError, reject_embedded_secrets};

const MAX_REQUEST_BYTES: usize = 1_048_576;

/// Opaque reference to a secret resolved only by an authorized host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretRefV1(pub String);

impl Validate for SecretRefV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_secret_ref("secret_ref", &self.0)
    }
}

/// Requested effect boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequestModeV1 {
    Observe,
    Propose,
    Execute,
}

/// Provider-neutral request accepted by a connector planner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorRequestV1 {
    pub schema: String,
    pub request_id: String,
    pub provider: String,
    pub capability: String,
    pub target: String,
    pub mode: RequestModeV1,
    #[serde(default)]
    pub secret_refs: Vec<SecretRefV1>,
    #[serde(default)]
    pub parameters: BTreeMap<String, Value>,
}

impl Validate for ConnectorRequestV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_schema("schema", &self.schema, REQUEST_SCHEMA_V1)?;
        validate_identifier("request_id", &self.request_id)?;
        validate_identifier("provider", &self.provider)?;
        validate_identifier("capability", &self.capability)?;
        validate_text("target", &self.target, 512)?;
        validate_secret_refs(&self.secret_refs)?;
        reject_embedded_secrets("parameters", &Value::Object(map_to_json(&self.parameters)))
    }
}

/// Parses a bounded, closed request document.
///
/// # Errors
///
/// Rejects oversized JSON, unknown fields, malformed JSON, and invalid values.
pub fn parse_request_json(source: &[u8]) -> Result<ConnectorRequestV1, ValidationError> {
    if source.len() > MAX_REQUEST_BYTES {
        return Err(ValidationError::new(
            "request",
            "document exceeds 1048576 bytes",
        ));
    }
    let request: ConnectorRequestV1 = serde_json::from_slice(source)
        .map_err(|_| ValidationError::new("request", "invalid closed v1 JSON"))?;
    request.validate()?;
    Ok(request)
}
