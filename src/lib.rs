#![forbid(unsafe_code)]
#![doc = "Versioned, provider-neutral wire contracts for Zixcel connectors."]

mod behavior_package;
mod error;
mod owner_recovery;
mod plan;
mod receipt;
mod request;
#[cfg(test)]
mod unit_tests;
mod validation;
mod workspace_action;
mod workspace_action_validation;
mod workspace_projection_receipt;

pub use behavior_package::{
    BehaviorArtifactRefV1, BehaviorEffectV1, BehaviorImplementationV1, BehaviorPackageV1,
    parse_behavior_package_json,
};
pub use error::{Validate, ValidationError};
pub use owner_recovery::{
    DEFAULT_MOVE_RECEIPT_RETENTION_SECONDS, OwnerRecoveryMethodV1, OwnerRecoveryRequestV1,
};
pub use plan::{ConnectorPlanV1, PlanEffectV1, PlanStepV1};
pub use receipt::{ConnectorReceiptV1, ReceiptArtifactV1, ReceiptStatusV1};
pub use request::{ConnectorRequestV1, RequestModeV1, SecretRefV1, parse_request_json};
pub use validation::reject_embedded_secrets;
pub use workspace_action::{
    WorkspaceActionRequestV1, WorkspaceActionV1, WorkspaceEffectV1, WorkspacePathV1,
    parse_workspace_action_json,
};
pub use workspace_projection_receipt::{
    WorkspaceActionReceiptV2, WorkspaceArtifactReferenceV1, WorkspaceProjectionReferenceV1,
    parse_workspace_receipt_v2_json,
};

/// Connector request schema URI.
pub const REQUEST_SCHEMA_V1: &str = "zixcel://contracts/connector-request/v1";
/// Deterministic connector plan schema URI.
pub const PLAN_SCHEMA_V1: &str = "zixcel://contracts/connector-plan/v1";
/// Connector execution receipt schema URI.
pub const RECEIPT_SCHEMA_V1: &str = "zixcel://contracts/connector-receipt/v1";
/// Filesystem and source-control action schema URI.
pub const WORKSPACE_ACTION_SCHEMA_V1: &str = "zixcel://contracts/workspace-action/v1";
/// Projection-bound filesystem and source-control receipt schema URI.
pub const WORKSPACE_RECEIPT_SCHEMA_V2: &str = "zixcel://contracts/workspace-receipt/v2";
/// Secret-free declaration used to request an owner recovery ceremony.
pub const OWNER_RECOVERY_REQUEST_SCHEMA_V1: &str = "zixcel://contracts/owner-recovery-request/v1";
/// Exact artifact-backed behavior implementations available for dependency injection.
pub const BEHAVIOR_PACKAGE_SCHEMA_V1: &str = "zixcel://contracts/behavior/package/v1";
