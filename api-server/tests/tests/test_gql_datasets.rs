use async_graphql::*;
use std::path::PathBuf;

use kamu::domain::*;
use kamu::infra;
use kamu::testing::MetadataFactory;
use opendatafabric::*;

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
        .execute("{ datasets { byId (datasetId: \"did:odf:z4k88e8n8Je6fC9Lz9FHrZ7XGsikEyBwTwtMBzxp4RH9pbWn4UM\") { name } } }")
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
    let (dataset_handle, _) = metadata_repo
        .add_dataset(
            MetadataFactory::dataset_snapshot()
                .name("foo")
                .kind(DatasetKind::Root)
                .push_event(MetadataFactory::set_polling_source().build())
                .build(),
        )
        .unwrap();

    let schema = kamu_api_server::gql::schema(cat);
    let res = schema
        .execute(format!(
            "{{ datasets {{ byId (datasetId: \"{}\") {{ name }} }} }}",
            dataset_handle.id
        ))
        .await;
    assert!(res.is_ok());
    assert_eq!(
        res.data,
        value!({
            "datasets": {
                "byId": {
                    "name": "foo",
                }
            }
        })
    );
}
