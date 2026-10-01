use std::collections::BTreeMap;

use serde_json::Value;

use crate::{SecretRefV1, ValidationError};

pub(crate) fn map_to_json(map: &BTreeMap<String, Value>) -> serde_json::Map<String, Value> {
    map.iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

pub(crate) fn validate_secret_refs(secret_refs: &[SecretRefV1]) -> Result<(), ValidationError> {
    if secret_refs.len() > 16 {
        return Err(ValidationError::new(
            "secret_refs",
            "must contain at most 16 entries",
        ));
    }
    for (index, secret_ref) in secret_refs.iter().enumerate() {
        validate_secret_ref(&format!("secret_refs[{index}]"), &secret_ref.0)?;
    }
    Ok(())
}

pub(crate) fn validate_secret_ref(field: &str, value: &str) -> Result<(), ValidationError> {
    let Some(path) = value.strip_prefix("secret://") else {
        return Err(ValidationError::new(
            field,
            "must use the secret:// reference scheme",
        ));
    };
    if path.len() < 3
        || path.len() > 240
        || !path.contains('/')
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains("//")
        || path.contains(['?', '#'])
        || path.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return Err(ValidationError::new(
            field,
            "contains an invalid secret reference path",
        ));
    }
    Ok(())
}

pub(crate) fn validate_schema(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            format!("expected {expected}, got {actual}"),
        ))
    }
}

pub(crate) fn validate_identifier(field: &str, value: &str) -> Result<(), ValidationError> {
    validate_text(field, value, 128)?;
    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
    }) {
        return Err(ValidationError::new(
            field,
            "must be a lowercase ASCII identifier (a-z, 0-9, -, _, .)",
        ));
    }
    Ok(())
}

pub(crate) fn validate_digest(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            "must be a 64-character lowercase hexadecimal SHA-256 digest",
        ))
    }
}

pub(crate) fn validate_text(
    field: &str,
    value: &str,
    maximum: usize,
) -> Result<(), ValidationError> {
    if value.trim().is_empty() || value.len() > maximum {
        Err(ValidationError::new(
            field,
            format!("must contain 1..={maximum} bytes"),
        ))
    } else {
        Ok(())
    }
}

/// Rejects credential-shaped keys recursively in extension JSON.
///
/// # Errors
///
/// Returns the path of the first forbidden key or excessive nesting.
pub fn reject_embedded_secrets(path: &str, value: &Value) -> Result<(), ValidationError> {
    reject_value(path, value, 0)
}

fn reject_value(path: &str, value: &Value, depth: usize) -> Result<(), ValidationError> {
    if depth > 32 {
        return Err(ValidationError::new(path, "JSON nesting exceeds 32 levels"));
    }
    match value {
        Value::Object(map) => {
            if map.len() > 128 {
                return Err(ValidationError::new(path, "object exceeds 128 fields"));
            }
            for (key, nested) in map {
                let child = format!("{path}.{key}");
                if forbidden_key(key) {
                    return Err(ValidationError::new(
                        child,
                        "embedded credentials are forbidden; use secret_refs",
                    ));
                }
                reject_value(&child, nested, depth + 1)?;
            }
        }
        Value::Array(items) => {
            if items.len() > 1_024 {
                return Err(ValidationError::new(path, "array exceeds 1024 entries"));
            }
            for (index, nested) in items.iter().enumerate() {
                reject_value(&format!("{path}[{index}]"), nested, depth + 1)?;
            }
        }
        Value::String(text) if text.len() > 65_536 => {
            return Err(ValidationError::new(path, "string exceeds 65536 bytes"));
        }
        _ => {}
    }
    Ok(())
}

fn forbidden_key(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        normalized.as_str(),
        "password"
            | "passwd"
            | "token"
            | "accesstoken"
            | "refreshtoken"
            | "clientsecret"
            | "privatekey"
            | "apikey"
            | "credential"
            | "credentials"
            | "secret"
    )
}
