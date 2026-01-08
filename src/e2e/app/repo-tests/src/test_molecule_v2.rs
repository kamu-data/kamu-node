// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use async_graphql::{Request, Variables};
use base64::Engine as _;
use indoc::indoc;
use kamu_cli_e2e_common::{KamuApiServerClient, KamuApiServerClientExt};
use serde_json::{Value, json};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_data_room_quota_exceeded(
    mut kamu_api_server_client: KamuApiServerClient,
) {
    const ACCOUNT: &str = "molecule";
    const PASSWORD: &str = "molecule.dev";

    kamu_api_server_client
        .auth()
        .login_with_password(ACCOUNT, PASSWORD)
        .await;

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "1201";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");

    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "quota-room-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    let create_project_json = create_project_res
        .data
        .clone()
        .into_json()
        .unwrap_or_default();
    assert_eq!(
        create_project_json["molecule"]["v2"]["createProject"]["isSuccess"],
        json!(true),
        "Project creation failed: {create_project_res:?}"
    );

    kamu_api_server_client
        .account()
        .set_account_quota_bytes(ACCOUNT, 1)
        .await;

    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(b"hello");
    let upload_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(DATA_ROOM_UPLOAD).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/quota.txt",
                "content": encoded,
            }))),
        )
        .await;

    assert!(
        upload_res.errors.is_empty(),
        "Upload mutation failed: {upload_res:?}"
    );
    let payload = upload_res.data.into_json().unwrap_or_default()["molecule"]["v2"]["project"]
        ["dataRoom"]["uploadFile"]
        .clone();
    assert_eq!(payload["isSuccess"], json!(false), "{payload:?}");
    let msg = payload["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("Quota exceeded"),
        "unexpected message: {payload:?}"
    );
    assert_eq!(payload["__typename"], json!("MoleculeQuotaExceeded"));
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_announcements_quota_exceeded(
    mut kamu_api_server_client: KamuApiServerClient,
) {
    const ACCOUNT: &str = "molecule";
    const PASSWORD: &str = "molecule.dev";

    kamu_api_server_client
        .auth()
        .login_with_password(ACCOUNT, PASSWORD)
        .await;

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "1202";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");

    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "quota-announce-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    let create_project_json = create_project_res
        .data
        .clone()
        .into_json()
        .unwrap_or_default();
    assert_eq!(
        create_project_json["molecule"]["v2"]["createProject"]["isSuccess"],
        json!(true),
        "Project creation failed: {create_project_res:?}"
    );

    kamu_api_server_client
        .account()
        .set_account_quota_bytes(ACCOUNT, 1)
        .await;

    let announcement_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_ANNOUNCEMENT).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "headline": "quota",
                "body": "exceeded",
                "attachments": [],
                "accessLevel": "public",
                "changeBy": "did:ethr:0x43f3F090af7fF638ad0EfD64c5354B6945fE75BC",
                "categories": ["news"],
                "tags": ["quota"],
            }))),
        )
        .await;

    assert!(
        announcement_res.errors.is_empty(),
        "Announcement mutation failed: {announcement_res:?}"
    );
    let payload = announcement_res.data.into_json().unwrap_or_default()["molecule"]["v2"]
        ["project"]["announcements"]["create"]
        .clone();
    assert_eq!(payload["isSuccess"], json!(false), "{payload:?}");
    let msg = payload["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("Quota exceeded"),
        "unexpected message: {payload:?}"
    );
    // Announcement mutation surfaces a specific error union variant
    assert_eq!(
        payload["__typename"],
        json!("CreateAnnouncementErrorQuotaExceeded"),
        "{payload:?}"
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_activity_change_by_for_remove(
    mut kamu_api_server_client: KamuApiServerClient,
) {
    const USER_ACCOUNT: &str = "molecule";
    const USER_PASSWORD: &str = "molecule.dev";
    const USER_DID: &str = "did:ethr:0xE2eActivityUser";
    const ADMIN_DID: &str = "did:ethr:0xE2eActivityAdmin";

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "1300";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");

    let path = "/activity-entry.txt";
    let tag = "e2e-activity-tag";
    let category = "e2e-activity-category";

    kamu_api_server_client
        .auth()
        .login_with_password(USER_ACCOUNT, USER_PASSWORD)
        .await;

    // Create project
    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "activity-remove-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    assert!(
        create_project_res.errors.is_empty(),
        "Project creation failed: {create_project_res:?}"
    );
    let create_payload = create_project_res
        .data
        .clone()
        .into_json()
        .unwrap_or_default();
    assert_eq!(
        create_payload["molecule"]["v2"]["createProject"]["isSuccess"],
        json!(true),
        "{create_payload:?}"
    );

    // Upload initial file (change_by = user)
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(b"hello world");
    let upload_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_VERSIONED_FILE).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": path,
                "content": encoded,
                "contentType": "text/plain",
                "changeBy": USER_DID,
                "accessLevel": "public",
                "description": "initial",
                "categories": [category],
                "tags": [tag],
                "contentText": "hello world",
                "encryptionMetadata": Value::Null,
            }))),
        )
        .await;
    assert!(
        upload_res.errors.is_empty(),
        "Upload mutation failed: {upload_res:?}"
    );
    let upload_json = upload_res.data.into_json().unwrap_or_default();
    let file_ref = upload_json["molecule"]["v2"]["project"]["dataRoom"]["uploadFile"]["entry"]
        ["ref"]
        .as_str()
        .expect("ref missing")
        .to_string();

    // Update metadata (change_by = user)
    let update_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(UPDATE_METADATA_QUERY).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "ref": file_ref,
                "expectedHead": Value::Null,
                "changeBy": USER_DID,
                "accessLevel": "public",
                "description": "updated description",
                "categories": [category],
                "tags": [tag],
                "contentText": "updated",
            }))),
        )
        .await;
    assert!(
        update_res.errors.is_empty(),
        "Update metadata failed: {update_res:?}"
    );
    let update_json = update_res.data.into_json().unwrap_or_default();
    assert_eq!(
        update_json["molecule"]["v2"]["project"]["dataRoom"]["updateFileMetadata"]["isSuccess"],
        json!(true),
        "{update_json:?}"
    );

    // Remove entry with admin changeBy (still authorized as project owner)
    let remove_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(REMOVE_ENTRY_QUERY).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": path,
                "changeBy": ADMIN_DID,
            }))),
        )
        .await;
    assert!(
        remove_res.errors.is_empty(),
        "Remove entry failed: {remove_res:?}"
    );
    let remove_json = remove_res.data.into_json().unwrap_or_default();
    assert_eq!(
        remove_json["molecule"]["v2"]["project"]["dataRoom"]["removeEntry"]["isSuccess"],
        json!(true),
        "{remove_json:?}"
    );

    // Project activity (should surface admin changeBy on remove)
    let project_activity = kamu_api_server_client
        .graphql_api_call_ex(Request::new(LIST_PROJECT_ACTIVITY_QUERY).variables(
            Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "filters": {
                    "byTags": [tag],
                },
            })),
        ))
        .await;
    assert!(
        project_activity.errors.is_empty(),
        "Project activity query failed: {project_activity:?}"
    );
    let project_nodes = project_activity.data.into_json().unwrap_or_default()["molecule"]["v2"]
        ["project"]["activity"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        project_nodes.len(),
        3,
        "Unexpected project activity: {project_nodes:?}"
    );
    assert_eq!(
        project_nodes[0]["__typename"],
        json!("MoleculeActivityFileRemovedV2"),
        "{project_nodes:?}"
    );
    assert_eq!(
        project_nodes[0]["entry"]["changeBy"],
        json!(ADMIN_DID),
        "Project remove changeBy mismatch: {project_nodes:?}"
    );
    assert_eq!(
        project_nodes[1]["entry"]["changeBy"],
        json!(USER_DID),
        "Update changeBy mismatch: {project_nodes:?}"
    );
    assert_eq!(
        project_nodes[2]["entry"]["changeBy"],
        json!(USER_DID),
        "Add changeBy mismatch: {project_nodes:?}"
    );

    // Global activity should mirror the same changeBy values
    let global_activity = kamu_api_server_client
        .graphql_api_call_ex(Request::new(LIST_GLOBAL_ACTIVITY_QUERY).variables(
            Variables::from_json(json!({
                "filters": {
                    "byTags": [tag],
                },
            })),
        ))
        .await;
    assert!(
        global_activity.errors.is_empty(),
        "Global activity query failed: {global_activity:?}"
    );
    let global_nodes = global_activity.data.into_json().unwrap_or_default()["molecule"]["v2"]
        ["activity"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        global_nodes.len(),
        3,
        "Unexpected global activity: {global_nodes:?}"
    );
    assert_eq!(
        global_nodes[0]["__typename"],
        json!("MoleculeActivityFileRemovedV2"),
        "{global_nodes:?}"
    );
    assert_eq!(
        global_nodes[0]["entry"]["changeBy"],
        json!(ADMIN_DID),
        "Global remove changeBy mismatch: {global_nodes:?}"
    );
    assert_eq!(
        global_nodes[1]["entry"]["changeBy"],
        json!(USER_DID),
        "Global update changeBy mismatch: {global_nodes:?}"
    );
    assert_eq!(
        global_nodes[2]["entry"]["changeBy"],
        json!(USER_DID),
        "Global add changeBy mismatch: {global_nodes:?}"
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_data_room_operations(
    mut kamu_api_server_client: KamuApiServerClient,
) {
    const ACCOUNT: &str = "molecule";
    const PASSWORD: &str = "molecule.dev";
    const USER_DID: &str = "did:ethr:0xE2eDataRoomUser";

    kamu_api_server_client
        .auth()
        .login_with_password(ACCOUNT, PASSWORD)
        .await;

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "2001";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");

    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "data-room-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    assert!(
        create_project_res.errors.is_empty(),
        "Project creation failed: {create_project_res:?}"
    );

    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(b"hello data room");
    let upload_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_VERSIONED_FILE).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/foo.txt",
                "content": encoded,
                "contentType": "text/plain",
                "changeBy": USER_DID,
                "accessLevel": "public",
                "description": "initial",
                "categories": ["e2e-category"],
                "tags": ["e2e-tag"],
                "contentText": "hello data room",
                "encryptionMetadata": Value::Null,
            }))),
        )
        .await;
    assert!(
        upload_res.errors.is_empty(),
        "Upload mutation failed: {upload_res:?}"
    );
    let upload_json = upload_res.data.into_json().unwrap_or_default();
    assert_eq!(
        upload_json["molecule"]["v2"]["project"]["dataRoom"]["uploadFile"]["isSuccess"],
        json!(true),
        "{upload_json:?}"
    );
    let file_ref = upload_json["molecule"]["v2"]["project"]["dataRoom"]["uploadFile"]["entry"]
        ["ref"]
        .as_str()
        .expect("ref missing")
        .to_string();

    let update_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(UPDATE_METADATA_QUERY).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "ref": file_ref,
                "expectedHead": Value::Null,
                "changeBy": USER_DID,
                "accessLevel": "public",
                "description": "updated",
                "categories": ["e2e-category"],
                "tags": ["e2e-tag"],
                "contentText": "updated",
            }))),
        )
        .await;
    assert!(
        update_res.errors.is_empty(),
        "Update metadata failed: {update_res:?}"
    );
    let update_json = update_res.data.into_json().unwrap_or_default();
    assert_eq!(
        update_json["molecule"]["v2"]["project"]["dataRoom"]["updateFileMetadata"]["isSuccess"],
        json!(true),
        "{update_json:?}"
    );

    let move_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(MOVE_ENTRY_QUERY).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "fromPath": "/foo.txt",
                "toPath": "/moved/foo.txt",
                "changeBy": USER_DID,
            }))),
        )
        .await;
    assert!(
        move_res.errors.is_empty(),
        "Move entry failed: {move_res:?}"
    );
    let move_json = move_res.data.into_json().unwrap_or_default();
    assert_eq!(
        move_json["molecule"]["v2"]["project"]["dataRoom"]["moveEntry"]["isSuccess"],
        json!(true),
        "{move_json:?}"
    );

    let list_entries = kamu_api_server_client
        .graphql_api_call_ex(Request::new(LIST_DATA_ROOM_ENTRIES_QUERY).variables(
            Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "filters": Value::Null,
            })),
        ))
        .await;
    assert!(
        list_entries.errors.is_empty(),
        "List entries failed: {list_entries:?}"
    );
    let list_json = list_entries.data.into_json().unwrap_or_default();
    assert_eq!(
        list_json["molecule"]["v2"]["project"]["dataRoom"]["latest"]["entries"]["totalCount"],
        json!(1),
        "{list_json:?}"
    );
    assert_eq!(
        list_json["molecule"]["v2"]["project"]["dataRoom"]["latest"]["entries"]["nodes"][0]["path"],
        json!("/moved/foo.txt"),
        "{list_json:?}"
    );

    let remove_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(REMOVE_ENTRY_QUERY).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/moved/foo.txt",
                "changeBy": USER_DID,
            }))),
        )
        .await;
    assert!(
        remove_res.errors.is_empty(),
        "Remove entry failed: {remove_res:?}"
    );
    let remove_json = remove_res.data.into_json().unwrap_or_default();
    assert_eq!(
        remove_json["molecule"]["v2"]["project"]["dataRoom"]["removeEntry"]["isSuccess"],
        json!(true),
        "{remove_json:?}"
    );

    let list_entries = kamu_api_server_client
        .graphql_api_call_ex(Request::new(LIST_DATA_ROOM_ENTRIES_QUERY).variables(
            Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "filters": Value::Null,
            })),
        ))
        .await;
    assert!(
        list_entries.errors.is_empty(),
        "List entries failed: {list_entries:?}"
    );
    let list_json = list_entries.data.into_json().unwrap_or_default();
    assert_eq!(
        list_json["molecule"]["v2"]["project"]["dataRoom"]["latest"]["entries"]["totalCount"],
        json!(0),
        "{list_json:?}"
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_announcements_operations(
    mut kamu_api_server_client: KamuApiServerClient,
) {
    const ACCOUNT: &str = "molecule";
    const PASSWORD: &str = "molecule.dev";
    const USER_DID: &str = "did:ethr:0xE2eAnnouncementUser";

    kamu_api_server_client
        .auth()
        .login_with_password(ACCOUNT, PASSWORD)
        .await;

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "2002";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");

    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "announce-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    assert!(
        create_project_res.errors.is_empty(),
        "Project creation failed: {create_project_res:?}"
    );

    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(b"announcement file");
    let upload_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_VERSIONED_FILE).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/attachment.txt",
                "content": encoded,
                "contentType": "text/plain",
                "changeBy": USER_DID,
                "accessLevel": "public",
                "description": "attachment",
                "categories": ["announce-category"],
                "tags": ["announce-tag"],
                "contentText": "announcement file",
                "encryptionMetadata": Value::Null,
            }))),
        )
        .await;
    assert!(
        upload_res.errors.is_empty(),
        "Upload mutation failed: {upload_res:?}"
    );
    let upload_json = upload_res.data.into_json().unwrap_or_default();
    let attachment_ref = upload_json["molecule"]["v2"]["project"]["dataRoom"]["uploadFile"]
        ["entry"]["ref"]
        .as_str()
        .expect("ref missing")
        .to_string();

    let announcement_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_ANNOUNCEMENT).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "headline": "E2E announcement",
                "body": "Announcement body",
                "attachments": [attachment_ref],
                "accessLevel": "public",
                "changeBy": USER_DID,
                "categories": ["announce-category"],
                "tags": ["announce-tag"],
            }))),
        )
        .await;
    assert!(
        announcement_res.errors.is_empty(),
        "Announcement mutation failed: {announcement_res:?}"
    );
    let announcement_json = announcement_res.data.into_json().unwrap_or_default();
    assert_eq!(
        announcement_json["molecule"]["v2"]["project"]["announcements"]["create"]["isSuccess"],
        json!(true),
        "{announcement_json:?}"
    );

    let list_announcements = kamu_api_server_client
        .graphql_api_call_ex(Request::new(LIST_ANNOUNCEMENTS_QUERY).variables(
            Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "filters": Value::Null,
            })),
        ))
        .await;
    assert!(
        list_announcements.errors.is_empty(),
        "List announcements failed: {list_announcements:?}"
    );
    let list_json = list_announcements.data.into_json().unwrap_or_default();
    assert_eq!(
        list_json["molecule"]["v2"]["project"]["announcements"]["tail"]["totalCount"],
        json!(1),
        "{list_json:?}"
    );
    assert_eq!(
        list_json["molecule"]["v2"]["project"]["announcements"]["tail"]["nodes"][0]["headline"],
        json!("E2E announcement"),
        "{list_json:?}"
    );
    assert_eq!(
        list_json["molecule"]["v2"]["project"]["announcements"]["tail"]["nodes"][0]["attachments"]
            [0]["ref"],
        json!(attachment_ref),
        "{list_json:?}"
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_activity(mut kamu_api_server_client: KamuApiServerClient) {
    const ACCOUNT: &str = "molecule";
    const PASSWORD: &str = "molecule.dev";
    const USER_1: &str = "did:ethr:0xE2eActivityUser1";
    const USER_2: &str = "did:ethr:0xE2eActivityUser2";

    kamu_api_server_client
        .auth()
        .login_with_password(ACCOUNT, PASSWORD)
        .await;

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "2003";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");
    let tag = "e2e-activity-tag";

    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "activity-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    assert!(
        create_project_res.errors.is_empty(),
        "Project creation failed: {create_project_res:?}"
    );

    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(b"activity file");
    let upload_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_VERSIONED_FILE).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/activity.txt",
                "content": encoded,
                "contentType": "text/plain",
                "changeBy": USER_1,
                "accessLevel": "public",
                "description": "activity",
                "categories": ["activity-category"],
                "tags": [tag],
                "contentText": "activity file",
                "encryptionMetadata": Value::Null,
            }))),
        )
        .await;
    assert!(
        upload_res.errors.is_empty(),
        "Upload mutation failed: {upload_res:?}"
    );

    let announcement_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_ANNOUNCEMENT).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "headline": "Activity announcement",
                "body": "Announcement body",
                "attachments": [],
                "accessLevel": "public",
                "changeBy": USER_2,
                "categories": ["activity-category"],
                "tags": [tag],
            }))),
        )
        .await;
    assert!(
        announcement_res.errors.is_empty(),
        "Announcement mutation failed: {announcement_res:?}"
    );

    let remove_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(REMOVE_ENTRY_QUERY).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/activity.txt",
                "changeBy": USER_2,
            }))),
        )
        .await;
    assert!(
        remove_res.errors.is_empty(),
        "Remove entry failed: {remove_res:?}"
    );

    let project_activity = kamu_api_server_client
        .graphql_api_call_ex(Request::new(LIST_PROJECT_ACTIVITY_QUERY).variables(
            Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "filters": {
                    "byTags": [tag],
                },
            })),
        ))
        .await;
    assert!(
        project_activity.errors.is_empty(),
        "Project activity query failed: {project_activity:?}"
    );
    let nodes = project_activity.data.into_json().unwrap_or_default()["molecule"]["v2"]["project"]
        ["activity"]["nodes"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        nodes
            .iter()
            .any(|node| node["__typename"] == "MoleculeActivityFileAddedV2"),
        "Missing file added activity: {nodes:?}"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node["__typename"] == "MoleculeActivityAnnouncementV2"),
        "Missing announcement activity: {nodes:?}"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node["__typename"] == "MoleculeActivityFileRemovedV2"),
        "Missing file removed activity: {nodes:?}"
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_molecule_v2_search(mut kamu_api_server_client: KamuApiServerClient) {
    const ACCOUNT: &str = "molecule";
    const PASSWORD: &str = "molecule.dev";
    const USER_DID: &str = "did:ethr:0xE2eSearchUser";

    kamu_api_server_client
        .auth()
        .login_with_password(ACCOUNT, PASSWORD)
        .await;

    const IPNFT_ADDRESS: &str = "0xcaD88677CA87a7815728C72D74B4ff4982d54Fc1";
    const IPNFT_TOKEN_ID: &str = "2004";
    let ipnft_uid = format!("{IPNFT_ADDRESS}_{IPNFT_TOKEN_ID}");

    let create_project_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_PROJECT).variables(Variables::from_json(json!({
                "ipnftSymbol": "search-e2e",
                "ipnftUid": ipnft_uid,
                "ipnftAddress": IPNFT_ADDRESS,
                "ipnftTokenId": IPNFT_TOKEN_ID,
            }))),
        )
        .await;
    assert!(
        create_project_res.errors.is_empty(),
        "Project creation failed: {create_project_res:?}"
    );

    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(b"kamu search e2e");
    let upload_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_VERSIONED_FILE).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "path": "/search.txt",
                "content": encoded,
                "contentType": "text/plain",
                "changeBy": USER_DID,
                "accessLevel": "public",
                "description": "search file",
                "categories": ["search-category"],
                "tags": ["search-tag"],
                "contentText": "kamu search e2e",
                "encryptionMetadata": Value::Null,
            }))),
        )
        .await;
    assert!(
        upload_res.errors.is_empty(),
        "Upload mutation failed: {upload_res:?}"
    );

    let announcement_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(CREATE_ANNOUNCEMENT).variables(Variables::from_json(json!({
                "ipnftUid": ipnft_uid,
                "headline": "Searchable announcement",
                "body": "kamu search e2e",
                "attachments": [],
                "accessLevel": "public",
                "changeBy": USER_DID,
                "categories": ["search-category"],
                "tags": ["search-tag"],
            }))),
        )
        .await;
    assert!(
        announcement_res.errors.is_empty(),
        "Announcement mutation failed: {announcement_res:?}"
    );

    let search_res = kamu_api_server_client
        .graphql_api_call_ex(
            Request::new(SEARCH_QUERY).variables(Variables::from_json(json!({
                "prompt": "kamu search",
                "filters": {
                    "byIpnftUids": [ipnft_uid],
                },
            }))),
        )
        .await;
    assert!(
        search_res.errors.is_empty(),
        "Search query failed: {search_res:?}"
    );
    let search_json = search_res.data.into_json().unwrap_or_default();
    let total_count = search_json["molecule"]["v2"]["search"]["totalCount"]
        .as_u64()
        .unwrap_or_default();
    assert!(
        total_count >= 1,
        "Unexpected search results: {search_json:?}"
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Queries
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

const CREATE_PROJECT: &str = indoc!(
    r#"
    mutation (
        $ipnftSymbol: String!,
        $ipnftUid: String!,
        $ipnftAddress: String!,
        $ipnftTokenId: Int!,
    ) {
        molecule {
            v2 {
                createProject(
                    ipnftSymbol: $ipnftSymbol,
                    ipnftUid: $ipnftUid,
                    ipnftAddress: $ipnftAddress,
                    ipnftTokenId: $ipnftTokenId,
                ) {
                    isSuccess
                    message
                    __typename
                }
            }
        }
    }
    "#
);

const DATA_ROOM_UPLOAD: &str = indoc!(
    r#"
    mutation ($ipnftUid: String!, $path: CollectionPath!, $content: Base64Usnp!) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            dataRoom {
              uploadFile(
                path: $path
                content: $content
                contentType: "text/plain"
                changeBy: "did:ethr:0x43f3F090af7fF638ad0EfD64c5354B6945fE75BC"
                accessLevel: "public"
              ) {
                isSuccess
                message
                __typename
              }
            }
          }
        }
      }
    }
    "#
);

// TODO: find a way to output tags/categories
const LIST_GLOBAL_ACTIVITY_QUERY: &str = indoc!(
    r#"
    query ($filters: MoleculeProjectActivityFilters) {
      molecule {
        v2 {
          activity(filters: $filters) {
            nodes {
              ... on MoleculeActivityFileAddedV2 {
                __typename
                entry {
                  path
                  ref
                  accessLevel
                  changeBy
                }
              }
              ... on MoleculeActivityFileUpdatedV2 {
                __typename
                entry {
                  path
                  ref
                  accessLevel
                  changeBy
                }
              }
              ... on MoleculeActivityFileRemovedV2 {
                __typename
                entry {
                  path
                  ref
                  accessLevel
                  changeBy
                }
              }
              ... on MoleculeActivityAnnouncementV2 {
                __typename
                announcement {
                  id
                  headline
                  body
                  attachments {
                    path
                    ref
                  }
                  accessLevel
                  changeBy
                  categories
                  tags
                }
              }
            }
          }
        }
      }
    }
    "#
);

const LIST_PROJECT_ACTIVITY_QUERY: &str = indoc!(
    r#"
    query ($ipnftUid: String!, $filters: MoleculeProjectActivityFilters) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            activity(filters: $filters) {
              nodes {
                ... on MoleculeActivityFileAddedV2 {
                  __typename
                  entry {
                    path
                    ref
                    accessLevel
                    changeBy
                  }
                }
                ... on MoleculeActivityFileUpdatedV2 {
                  __typename
                  entry {
                    path
                    ref
                    accessLevel
                    changeBy
                  }
                }
                ... on MoleculeActivityFileRemovedV2 {
                  __typename
                  entry {
                    path
                    ref
                    accessLevel
                    changeBy
                  }
                }
                ... on MoleculeActivityAnnouncementV2 {
                  __typename
                  announcement {
                    id
                    headline
                    body
                    attachments {
                      path
                      ref
                    }
                    accessLevel
                    changeBy
                    categories
                    tags
                  }
                }
              }
            }
          }
        }
      }
    }
    "#
);

const CREATE_ANNOUNCEMENT: &str = indoc!(
    r#"
    mutation ($ipnftUid: String!, $headline: String!, $body: String!, $attachments: [DatasetID!], $accessLevel: String!, $changeBy: String!, $categories: [String!]!, $tags: [String!]!) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            announcements {
              create(
                headline: $headline
                body: $body
                attachments: $attachments
                accessLevel: $accessLevel
                changeBy: $changeBy
                categories: $categories
                tags: $tags
              ) {
                isSuccess
                message
                __typename
              }
            }
          }
        }
      }
    }
    "#
);

const MOVE_ENTRY_QUERY: &str = indoc!(
    r#"
    mutation ($ipnftUid: String!, $fromPath: CollectionPathV2!, $toPath: CollectionPathV2!, $changeBy: String!) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            dataRoom {
              moveEntry(fromPath: $fromPath, toPath: $toPath, changeBy: $changeBy) {
                isSuccess
                message
              }
            }
          }
        }
      }
    }
    "#
);

const CREATE_VERSIONED_FILE: &str = indoc!(
    r#"
    mutation ($ipnftUid: String!, $path: CollectionPathV2!, $content: Base64Usnp!, $contentType: String!, $changeBy: String!, $accessLevel: String!, $description: String, $categories: [String!], $tags: [String!], $contentText: String, $encryptionMetadata: MoleculeEncryptionMetadataInput) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            dataRoom {
              uploadFile(
                path: $path
                content: $content
                contentType: $contentType
                changeBy: $changeBy
                accessLevel: $accessLevel
                description: $description
                categories: $categories
                tags: $tags
                contentText: $contentText
                encryptionMetadata: $encryptionMetadata
              ) {
                isSuccess
                message
                ... on MoleculeDataRoomFinishUploadFileResultSuccess {
                  entry {
                    ref
                    path
                    changeBy
                  }
                }
              }
            }
          }
        }
      }
    }
    "#
);

const UPDATE_METADATA_QUERY: &str = indoc!(
    r#"
    mutation ($ipnftUid: String!, $ref: DatasetID!, $expectedHead: String, $changeBy: String!, $accessLevel: String!, $description: String, $categories: [String!], $tags: [String!], $contentText: String) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            dataRoom {
              updateFileMetadata(
                ref: $ref
                expectedHead: $expectedHead
                accessLevel: $accessLevel
                changeBy: $changeBy
                description: $description
                categories: $categories
                tags: $tags
                contentText: $contentText
              ) {
                isSuccess
                message
                __typename
                ... on UpdateVersionErrorCasFailed {
                    expectedHead
                    actualHead
                }
              }
            }
          }
        }
      }
    }
    "#
);

const LIST_DATA_ROOM_ENTRIES_QUERY: &str = indoc!(
    r#"
    query ($ipnftUid: String!, $filters: MoleculeDataRoomEntriesFilters) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            dataRoom {
              latest {
                entries(filters: $filters) {
                  totalCount
                  nodes {
                    path
                    ref
                  }
                }
              }
            }
          }
        }
      }
    }
    "#
);

const LIST_ANNOUNCEMENTS_QUERY: &str = indoc!(
    r#"
    query ($ipnftUid: String!, $filters: MoleculeAnnouncementsFilters) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            announcements {
              tail(filters: $filters) {
                totalCount
                nodes {
                  id
                  headline
                  body
                  attachments {
                    path
                    ref
                  }
                  accessLevel
                  changeBy
                  categories
                  tags
                }
              }
            }
          }
        }
      }
    }
    "#
);

const SEARCH_QUERY: &str = indoc!(
    r#"
    query ($prompt: String!, $filters: MoleculeSemanticSearchFilters) {
      molecule {
        v2 {
          search(prompt: $prompt, filters: $filters) {
            totalCount
            nodes {
              ... on MoleculeSemanticSearchFoundDataRoomEntry {
                __typename
                entry {
                  project {
                    ipnftUid
                  }
                  path
                  ref
                  changeBy
                  accessLevel
                }
              }
              ... on MoleculeSemanticSearchFoundAnnouncement {
                __typename
                announcement {
                  project {
                    ipnftUid
                  }
                  id
                  headline
                  body
                  accessLevel
                  changeBy
                }
              }
            }
          }
        }
      }
    }
    "#
);

const REMOVE_ENTRY_QUERY: &str = indoc!(
    r#"
    mutation ($ipnftUid: String!, $path: CollectionPathV2!, $changeBy: String!) {
      molecule {
        v2 {
          project(ipnftUid: $ipnftUid) {
            dataRoom {
              removeEntry(path: $path, changeBy: $changeBy) {
                isSuccess
                message
              }
            }
          }
        }
      }
    }
    "#
);
