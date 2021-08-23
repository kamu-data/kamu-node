use async_graphql::*;
use std::path::PathBuf;

use kamu::domain::*;
use kamu::infra;
use kamu::testing::MetadataFactory;

#[test]
fn update_schema_dump() {
    let mut cat = dill::Catalog::new();
    cat.add_value(infra::MetadataRepositoryNull);
    cat.bind::<dyn MetadataRepository, infra::MetadataRepositoryNull>()
        .unwrap();
    let schema = kamu_api_server::gql::schema(cat).sdl();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/schema.gql");

    std::fs::write(path, schema).unwrap();
}

#[tokio::test]
async fn dataset_by_id_does_not_exist() {
    let mut cat = dill::Catalog::new();
    cat.add_value(infra::MetadataRepositoryNull);
    cat.bind::<dyn MetadataRepository, infra::MetadataRepositoryNull>()
        .unwrap();
    let schema = kamu_api_server::gql::schema(cat);
    let res = schema
        .execute("{ datasets { byId (id: \"test\") { id } } }")
        .await;
    assert_eq!(
        res.data,
        value!({
            "datasets": {
                "byId": null,
            }
        })
    );
}

#[tokio::test]
async fn dataset_by_id() {
    let tempdir = tempfile::tempdir().unwrap();

    let workspace_layout = infra::WorkspaceLayout::create(tempdir.path()).unwrap();

    let mut cat = dill::Catalog::new();
    cat.add_value(workspace_layout);
    cat.add::<infra::MetadataRepositoryImpl>();
    cat.bind::<dyn MetadataRepository, infra::MetadataRepositoryImpl>()
        .unwrap();

    let metadata_repo = cat.get_one::<dyn MetadataRepository>().unwrap();
    metadata_repo
        .add_dataset(
            MetadataFactory::dataset_snapshot()
                .id("foo")
                .source(MetadataFactory::dataset_source_root().build())
                .build(),
        )
        .unwrap();

    let schema = kamu_api_server::gql::schema(cat);
    let res = schema
        .execute("{ datasets { byId (id: \"foo\") { id } } }")
        .await;
    assert!(res.is_ok());
    assert_eq!(
        res.data,
        value!({
            "datasets": {
                "byId": {
                    "id": "foo",
                }
            }
        })
    );
}
