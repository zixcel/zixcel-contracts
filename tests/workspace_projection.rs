use std::collections::BTreeMap;

use zixcel_contracts::{
    ReceiptStatusV1, WORKSPACE_RECEIPT_SCHEMA_V2, WorkspaceActionReceiptV2,
    WorkspaceArtifactReferenceV1, WorkspaceProjectionReferenceV1, parse_workspace_receipt_v2_json,
};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn artifact() -> WorkspaceArtifactReferenceV1 {
    WorkspaceArtifactReferenceV1 {
        owner_id: "zixcel-filesystem".to_owned(),
        artifact_ref: "artifact-1".to_owned(),
        resolution_ref: "resolution-1".to_owned(),
        schema_id: "zixcel://projection/file-content/v1".to_owned(),
        media_type: "text/plain".to_owned(),
        digest_sha256: DIGEST.to_owned(),
        size_bytes: 42,
        classification: "internal".to_owned(),
    }
}

#[test]
fn successful_receipt_requires_an_owned_typed_projection_reference() {
    let receipt = WorkspaceActionReceiptV2 {
        schema: WORKSPACE_RECEIPT_SCHEMA_V2.to_owned(),
        receipt_id: "receipt-1".to_owned(),
        request_id: "request-1".to_owned(),
        request_digest_sha256: DIGEST.to_owned(),
        workspace_ref: "workspace-1".to_owned(),
        status: ReceiptStatusV1::Succeeded,
        observations: BTreeMap::new(),
        projection: Some(WorkspaceProjectionReferenceV1 {
            projection_id: "projection-1".to_owned(),
            revision: 5,
            artifact: artifact(),
        }),
        artifacts: vec![artifact()],
    };
    let wire = serde_json::to_vec(&receipt).expect("serialize");
    assert_eq!(parse_workspace_receipt_v2_json(&wire).unwrap(), receipt);

    let mut missing = receipt;
    missing.projection = None;
    assert!(parse_workspace_receipt_v2_json(&serde_json::to_vec(&missing).unwrap()).is_err());
}

#[test]
fn artifact_reference_rejects_copied_content_and_unknown_resolution_fields() {
    let mut value = serde_json::to_value(artifact()).expect("serialize");
    value.as_object_mut().expect("object").insert(
        "content".to_owned(),
        serde_json::json!("copied workspace body"),
    );
    assert!(serde_json::from_value::<WorkspaceArtifactReferenceV1>(value).is_err());
}
