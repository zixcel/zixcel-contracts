use serde::{Deserialize, Serialize};

use crate::validation::{validate_identifier, validate_schema};
use crate::{OWNER_RECOVERY_REQUEST_SCHEMA_V1, Validate, ValidationError};

/// Two common years expressed as 730 days. The owner may select another value.
pub const DEFAULT_MOVE_RECEIPT_RETENTION_SECONDS: u64 = 63_072_000;
const MIN_RECEIPT_RETENTION_SECONDS: u64 = 86_400;
const MAX_RECEIPT_RETENTION_SECONDS: u64 = 315_360_000;
const MAX_REQUEST_LIFETIME_SECONDS: u64 = 900;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OwnerRecoveryMethodV1 {
    MnemonicSeedPhrase,
}

/// Public, secret-free input for Zixcel recovery orchestration.
///
/// The mnemonic and derived key material are deliberately absent. Zixcel passes
/// this declaration to an explicitly selected Crowsi custody implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerRecoveryRequestV1 {
    pub schema: String,
    pub request_id: String,
    pub subject_ref: String,
    pub method: OwnerRecoveryMethodV1,
    pub custody_provider_ref: String,
    pub receipt_retention_seconds: u64,
    pub issued_at_epoch_s: u64,
    pub expires_at_epoch_s: u64,
}

impl Validate for OwnerRecoveryRequestV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_schema("schema", &self.schema, OWNER_RECOVERY_REQUEST_SCHEMA_V1)?;
        validate_identifier("request_id", &self.request_id)?;
        validate_identifier("subject_ref", &self.subject_ref)?;
        validate_identifier("custody_provider_ref", &self.custody_provider_ref)?;
        if self.receipt_retention_seconds < MIN_RECEIPT_RETENTION_SECONDS
            || self.receipt_retention_seconds > MAX_RECEIPT_RETENTION_SECONDS
        {
            return Err(ValidationError::new(
                "receipt_retention_seconds",
                "must be between one day and ten years",
            ));
        }
        if self.issued_at_epoch_s == 0
            || self.expires_at_epoch_s <= self.issued_at_epoch_s
            || self.expires_at_epoch_s - self.issued_at_epoch_s > MAX_REQUEST_LIFETIME_SECONDS
        {
            return Err(ValidationError::new(
                "expires_at_epoch_s",
                "recovery request lifetime must be at most fifteen minutes",
            ));
        }
        Ok(())
    }
}
