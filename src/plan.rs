use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::validation::{
    map_to_json, validate_identifier, validate_schema, validate_secret_refs, validate_text,
};
use crate::{
    PLAN_SCHEMA_V1, RequestModeV1, SecretRefV1, Validate, ValidationError, reject_embedded_secrets,
};

/// Effect that an executor may perform after separate authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanEffectV1 {
    None,
    Observe,
    Mutate,
}

/// One ordered, explicit connector plan step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanStepV1 {
    pub sequence: u32,
    pub action: String,
    pub target: String,
    pub effect: PlanEffectV1,
    pub network_required: bool,
}

/// Deterministic proposal consumed by a separately authorized executor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorPlanV1 {
    pub schema: String,
    pub plan_id: String,
    pub request_id: String,
    pub provider: String,
    pub connector: String,
    pub mode: RequestModeV1,
    pub steps: Vec<PlanStepV1>,
    #[serde(default)]
    pub secret_refs: Vec<SecretRefV1>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

impl Validate for ConnectorPlanV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_schema("schema", &self.schema, PLAN_SCHEMA_V1)?;
        validate_identifier("plan_id", &self.plan_id)?;
        validate_identifier("request_id", &self.request_id)?;
        validate_identifier("provider", &self.provider)?;
        validate_identifier("connector", &self.connector)?;
        if self.steps.is_empty() || self.steps.len() > 256 {
            return Err(ValidationError::new(
                "steps",
                "must contain 1..=256 plan steps",
            ));
        }
        for (index, step) in self.steps.iter().enumerate() {
            let expected = u32::try_from(index + 1)
                .map_err(|_| ValidationError::new("steps", "too many plan steps"))?;
            if step.sequence != expected {
                return Err(ValidationError::new(
                    format!("steps[{index}].sequence"),
                    format!("expected {expected}, got {}", step.sequence),
                ));
            }
            validate_identifier(&format!("steps[{index}].action"), &step.action)?;
            validate_text(&format!("steps[{index}].target"), &step.target, 512)?;
        }
        validate_secret_refs(&self.secret_refs)?;
        reject_embedded_secrets("extensions", &Value::Object(map_to_json(&self.extensions)))
    }
}
