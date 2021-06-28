use async_graphql::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use kamu::domain::*;
use kamu::infra;
use kamu_test::MetadataFactory;

#[test]
fn update_schema_dump() {
    let schema =
        kamu_api_server::gql::schema(Arc::new(Mutex::new(infra::MetadataRepositoryNull))).sdl();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/schema.gql");

    std::fs::write(path, schema).unwrap();
}

#[tokio::test]
async fn dataset_by_id_does_not_exist() {
    let schema = kamu_api_server::gql::schema(Arc::new(Mutex::new(infra::MetadataRepositoryNull)));
    let res = schema
        .execute("{ datasetById (id: \"test\") { id } }")
        .await;
    assert_eq!(
        res.data,
        value!({
            "datasetById": null,
        })
    );
}

#[tokio::test]
async fn dataset_by_id() {
    let tempdir = tempfile::tempdir().unwrap();

    let workspace_layout = infra::WorkspaceLayout::create(tempdir.path()).unwrap();
    let mut metadata_repo = infra::MetadataRepositoryImpl::new(&workspace_layout);

    metadata_repo
        .add_dataset(
            MetadataFactory::dataset_snapshot()
                .id("foo")
                .source(MetadataFactory::dataset_source_root().build())
                .build(),
        )
        .unwrap();

    let schema = kamu_api_server::gql::schema(Arc::new(Mutex::new(metadata_repo)));
    let res = schema.execute("{ datasetById (id: \"foo\") { id } }").await;
    assert!(res.is_ok());
    assert_eq!(
        res.data,
        value!({
            "datasetById": {
                "id": "foo",
            }
        })
    );
}
