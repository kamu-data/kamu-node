use async_graphql::*;
use std::path::PathBuf;

#[test]
fn update_schema_dump() {
    let schema = kamu_api_server::gql::schema().sdl();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/schema.gql");

    std::fs::write(path, schema).unwrap();
}

#[tokio::test]
async fn dataset_by_id() {
    let schema = kamu_api_server::gql::schema();
    let res = schema
        .execute("{ datasetById (id: \"test\") { id } }")
        .await;
    assert!(res.is_ok());
    assert_eq!(
        res.data,
        value!({
            "datasetById": {
                "id": "test",
            }
        })
    );
}
