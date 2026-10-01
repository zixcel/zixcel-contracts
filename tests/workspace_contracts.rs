use serde_json::json;
use zixcel_contracts::{
    WORKSPACE_ACTION_SCHEMA_V1, WorkspaceActionRequestV1, WorkspaceActionV1, WorkspaceEffectV1,
    WorkspacePathV1, parse_workspace_action_json,
};

fn request(action: WorkspaceActionV1) -> WorkspaceActionRequestV1 {
    WorkspaceActionRequestV1 {
        schema: WORKSPACE_ACTION_SCHEMA_V1.to_owned(),
        request_id: "request-1".to_owned(),
        workspace_ref: "workspace-1".to_owned(),
        grant_ref: "grant-1".to_owned(),
        action,
    }
}

#[test]
fn action_effect_is_closed_and_paths_are_workspace_relative() {
    let read = request(WorkspaceActionV1::ReadFile {
        path: WorkspacePathV1("src/lib.rs".to_owned()),
        maximum_bytes: 4_096,
    });
    assert_eq!(read.effect(), WorkspaceEffectV1::Observe);
    let encoded = serde_json::to_vec(&read).expect("serialize request");
    assert_eq!(parse_workspace_action_json(&encoded).unwrap(), read);

    let mutate = request(WorkspaceActionV1::Remove {
        path: WorkspacePathV1("target/cache".to_owned()),
        recursive: true,
    });
    assert_eq!(mutate.effect(), WorkspaceEffectV1::Mutate);

    for invalid in ["/etc/passwd", "../secret", "src//lib.rs", "src\\lib.rs"] {
        let mut value = serde_json::to_value(&read).unwrap();
        value["action"]["path"] = json!(invalid);
        assert!(parse_workspace_action_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}

#[test]
fn action_rejects_unknown_fields_and_invalid_digests() {
    let mut value = serde_json::to_value(request(WorkspaceActionV1::SourceApplyPatch {
        patch_digest_sha256: "ab".repeat(32),
        expected_head_sha256: "cd".repeat(32),
    }))
    .unwrap();
    value["transport"] = json!("local");
    assert!(parse_workspace_action_json(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("transport");
    value["action"]["patch_digest_sha256"] = json!("AB".repeat(32));
    assert!(parse_workspace_action_json(&serde_json::to_vec(&value).unwrap()).is_err());
}
