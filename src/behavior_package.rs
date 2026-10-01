use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::validation::{validate_digest, validate_text};
use crate::{BEHAVIOR_PACKAGE_SCHEMA_V1, Validate, ValidationError};

const MAX_DOCUMENT_BYTES: usize = 1_048_576;
const MAX_IMPLEMENTATIONS: usize = 128;
const MAX_REQUIREMENTS: usize = 64;

/// Immutable registry artifact that owns executable behavior.
///
/// The reference carries provenance only. It is not a path, loader handle,
/// credential or transport endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorArtifactRefV1 {
    pub registry_ref: String,
    pub package_ref: String,
    pub release: String,
    pub digest_sha256: String,
}

/// Effect disclosure for an implementation. sem-lang does not infer effects
/// from operation names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorEffectV1 {
    Observe,
    Infer,
    Mutate,
}

/// One implementation of a semantic behavior interface.
///
/// `behavior_ref`, input and output types are stable interfaces. The exact
/// operation target is the injected implementation. Requirements are other
/// behavior interfaces, never package names, so a project can replace an
/// implementation without changing language syntax.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorImplementationV1 {
    pub behavior_ref: String,
    pub operation_id: String,
    pub target: String,
    pub contract_revision: String,
    pub input_type_ref: String,
    pub output_type_ref: String,
    pub effect: BehaviorEffectV1,
    #[serde(default)]
    pub requires: Vec<String>,
}

/// A bounded package declaration used to inject behavior into a fixed
/// semantic language. It cannot contribute grammar or semantic definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorPackageV1 {
    pub schema: String,
    pub package_ref: String,
    pub package_revision: String,
    pub owner: String,
    pub artifact: BehaviorArtifactRefV1,
    pub implementations: Vec<BehaviorImplementationV1>,
}

impl Validate for BehaviorPackageV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.schema != BEHAVIOR_PACKAGE_SCHEMA_V1 {
            return Err(ValidationError::new(
                "schema",
                format!("expected {BEHAVIOR_PACKAGE_SCHEMA_V1}, got {}", self.schema),
            ));
        }
        validate_reference("package_ref", &self.package_ref)?;
        validate_reference("package_revision", &self.package_revision)?;
        validate_reference("owner", &self.owner)?;
        validate_reference("artifact.registry_ref", &self.artifact.registry_ref)?;
        validate_reference("artifact.package_ref", &self.artifact.package_ref)?;
        validate_text("artifact.release", &self.artifact.release, 64)?;
        validate_digest("artifact.digest_sha256", &self.artifact.digest_sha256)?;
        if self.implementations.is_empty() || self.implementations.len() > MAX_IMPLEMENTATIONS {
            return Err(ValidationError::new(
                "implementations",
                format!("must contain 1..={MAX_IMPLEMENTATIONS} entries"),
            ));
        }
        let mut behavior_refs = BTreeSet::new();
        for (index, implementation) in self.implementations.iter().enumerate() {
            let prefix = format!("implementations[{index}]");
            for (field, value) in [
                ("behavior_ref", &implementation.behavior_ref),
                ("operation_id", &implementation.operation_id),
                ("target", &implementation.target),
                ("contract_revision", &implementation.contract_revision),
                ("input_type_ref", &implementation.input_type_ref),
                ("output_type_ref", &implementation.output_type_ref),
            ] {
                validate_reference(&format!("{prefix}.{field}"), value)?;
            }
            if !behavior_refs.insert(&implementation.behavior_ref) {
                return Err(ValidationError::new(
                    format!("{prefix}.behavior_ref"),
                    "must be unique within the package",
                ));
            }
            if implementation.requires.len() > MAX_REQUIREMENTS {
                return Err(ValidationError::new(
                    format!("{prefix}.requires"),
                    format!("must contain at most {MAX_REQUIREMENTS} entries"),
                ));
            }
            let mut requirements = BTreeSet::new();
            for (requirement_index, requirement) in implementation.requires.iter().enumerate() {
                validate_reference(
                    &format!("{prefix}.requires[{requirement_index}]"),
                    requirement,
                )?;
                if requirement == &implementation.behavior_ref || !requirements.insert(requirement)
                {
                    return Err(ValidationError::new(
                        format!("{prefix}.requires[{requirement_index}]"),
                        "must be unique and must not require itself",
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Parse and validate one closed, bounded behavior package declaration.
///
/// # Errors
///
/// Rejects oversized documents, unknown fields, malformed hierarchical
/// references, duplicate implementations and embedded implementation details.
pub fn parse_behavior_package_json(source: &[u8]) -> Result<BehaviorPackageV1, ValidationError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(ValidationError::new(
            "behavior_package",
            "document exceeds 1048576 bytes",
        ));
    }
    let package: BehaviorPackageV1 = serde_json::from_slice(source).map_err(|_| {
        ValidationError::new("behavior_package", "invalid closed behavior package JSON")
    })?;
    package.validate()?;
    Ok(package)
}

fn validate_reference(field: &str, value: &str) -> Result<(), ValidationError> {
    validate_text(field, value, 256)?;
    if value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || value.split('/').any(|part| {
            part.is_empty()
                || !part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_' | b'.' | b'@')
                })
        })
    {
        return Err(ValidationError::new(
            field,
            "must be a canonical lowercase hierarchical reference",
        ));
    }
    Ok(())
}
