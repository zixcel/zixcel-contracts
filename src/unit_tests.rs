use serde_json::json;

use crate::reject_embedded_secrets;
use crate::validation::{validate_identifier, validate_secret_ref};

#[test]
fn identifiers_and_secret_references_fail_closed() {
    assert!(validate_identifier("id", "valid-id").is_ok());
    assert!(validate_identifier("id", "UPPER").is_err());
    assert!(validate_secret_ref("secret", "secret://provider/account").is_ok());
    assert!(validate_secret_ref("secret", "plain-text-secret").is_err());
}

#[test]
fn nested_credential_keys_are_rejected() {
    let value = json!({"safe": {"client_secret": "no"}});
    assert!(reject_embedded_secrets("parameters", &value).is_err());
}
