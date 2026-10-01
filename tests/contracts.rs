use std::collections::BTreeMap;

use serde_json::json;
use zixcel_contracts::{
    BEHAVIOR_PACKAGE_SCHEMA_V1, BehaviorArtifactRefV1, BehaviorEffectV1, BehaviorImplementationV1,
    BehaviorPackageV1, ConnectorPlanV1, ConnectorReceiptV1, ConnectorRequestV1,
    DEFAULT_MOVE_RECEIPT_RETENTION_SECONDS, OWNER_RECOVERY_REQUEST_SCHEMA_V1,
    OwnerRecoveryMethodV1, OwnerRecoveryRequestV1, PLAN_SCHEMA_V1, PlanEffectV1, PlanStepV1,
    RECEIPT_SCHEMA_V1, REQUEST_SCHEMA_V1, ReceiptArtifactV1, ReceiptStatusV1, RequestModeV1,
    SecretRefV1, Validate, parse_behavior_package_json, parse_request_json,
};

fn request() -> ConnectorRequestV1 {
    ConnectorRequestV1 {
        schema: REQUEST_SCHEMA_V1.to_owned(),
        request_id: "request-001".to_owned(),
        provider: "microsoft".to_owned(),
        capability: "sharepoint-observe".to_owned(),
        target: "site:operations".to_owned(),
        mode: RequestModeV1::Observe,
        secret_refs: vec![SecretRefV1(
            "secret://microsoft/graph/operations".to_owned(),
        )],
        parameters: BTreeMap::from([("depth".to_owned(), json!(2))]),
    }
}

#[test]
fn closed_request_round_trips_through_bounded_parser() {
    let encoded = serde_json::to_vec(&request()).expect("serialize");
    assert_eq!(parse_request_json(&encoded).expect("parse"), request());
    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).expect("value");
    unknown["credential_hint"] = json!("none");
    assert!(parse_request_json(&serde_json::to_vec(&unknown).expect("serialize")).is_err());
}

#[test]
fn rejects_credentials_and_unbounded_documents() {
    let mut value = request();
    value.parameters.insert(
        "authentication".to_owned(),
        json!({"client_secret": "must-not-be-here"}),
    );
    assert!(value.validate().is_err());
    assert!(parse_request_json(&vec![b' '; 1_048_577]).is_err());
}

#[test]
fn plan_steps_are_bounded_and_contiguous() {
    let plan = ConnectorPlanV1 {
        schema: PLAN_SCHEMA_V1.to_owned(),
        plan_id: "plan-001".to_owned(),
        request_id: "request-001".to_owned(),
        provider: "microsoft".to_owned(),
        connector: "zixcel-microsoft".to_owned(),
        mode: RequestModeV1::Propose,
        steps: vec![PlanStepV1 {
            sequence: 2,
            action: "validate-config".to_owned(),
            target: "operations".to_owned(),
            effect: PlanEffectV1::None,
            network_required: false,
        }],
        secret_refs: Vec::new(),
        extensions: BTreeMap::new(),
    };
    assert!(plan.validate().is_err());
}

#[test]
fn receipt_requires_unique_sha256_artifacts() {
    let artifact = ReceiptArtifactV1 {
        artifact_id: "artifact-001".to_owned(),
        media_type: "application/json".to_owned(),
        digest_sha256: "a".repeat(64),
    };
    let receipt = ConnectorReceiptV1 {
        schema: RECEIPT_SCHEMA_V1.to_owned(),
        receipt_id: "receipt-001".to_owned(),
        request_id: "request-001".to_owned(),
        plan_id: "plan-001".to_owned(),
        provider: "microsoft".to_owned(),
        status: ReceiptStatusV1::Succeeded,
        observations: BTreeMap::new(),
        artifacts: vec![artifact.clone(), artifact],
    };
    assert!(receipt.validate().is_err());
}

#[test]
fn owner_recovery_is_mnemonic_first_and_never_carries_the_phrase() {
    let request = OwnerRecoveryRequestV1 {
        schema: OWNER_RECOVERY_REQUEST_SCHEMA_V1.into(),
        request_id: "recovery-001".into(),
        subject_ref: "owner-001".into(),
        method: OwnerRecoveryMethodV1::MnemonicSeedPhrase,
        custody_provider_ref: "crowsi-owner-recovery".into(),
        receipt_retention_seconds: DEFAULT_MOVE_RECEIPT_RETENTION_SECONDS,
        issued_at_epoch_s: 100,
        expires_at_epoch_s: 1_000,
    };
    request.validate().expect("valid recovery request");
    let encoded = serde_json::to_value(&request).expect("serialize");
    assert_eq!(encoded["receipt_retention_seconds"], 63_072_000);
    assert!(encoded.get("mnemonic").is_none());
    assert!(encoded.get("seed").is_none());
    let mut too_short = request.clone();
    too_short.receipt_retention_seconds = 86_399;
    assert!(too_short.validate().is_err());
    let mut expired = request;
    expired.expires_at_epoch_s += 1;
    assert!(expired.validate().is_err());
}

#[test]
fn behavior_package_injects_exact_behavior_without_extending_language() {
    let package = BehaviorPackageV1 {
        schema: BEHAVIOR_PACKAGE_SCHEMA_V1.into(),
        package_ref: "zixcel/source/folder".into(),
        package_revision: "0.10.0".into(),
        owner: "zixcel/source".into(),
        artifact: BehaviorArtifactRefV1 {
            registry_ref: "zixcel/local/registry".into(),
            package_ref: "zixcel/source/folder".into(),
            release: "0.10.0".into(),
            digest_sha256: "a".repeat(64),
        },
        implementations: vec![BehaviorImplementationV1 {
            behavior_ref: "source/folder/observe".into(),
            operation_id: "source/folder/observe".into(),
            target: "source/folder".into(),
            contract_revision: "0.10.0".into(),
            input_type_ref: "source/folder/reference".into(),
            output_type_ref: "source/entry/collection".into(),
            effect: BehaviorEffectV1::Observe,
            requires: Vec::new(),
        }],
    };
    let bytes = serde_json::to_vec(&package).expect("serialize");
    assert_eq!(parse_behavior_package_json(&bytes).expect("parse"), package);

    let mut grammar_extension = serde_json::to_value(&package).expect("value");
    grammar_extension["grammar"] = serde_json::json!({"keyword": "folder"});
    assert!(
        parse_behavior_package_json(&serde_json::to_vec(&grammar_extension).expect("serialize"))
            .is_err()
    );

    let mut self_cycle = package;
    self_cycle.implementations[0].requires = vec!["source/folder/observe".into()];
    assert!(self_cycle.validate().is_err());
}
