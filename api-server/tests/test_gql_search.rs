use async_graphql::*;

use kamu::domain::*;
use kamu::infra;
use kamu_test::MetadataFactory;

#[tokio::test]
async fn query() {
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
        .execute(
            "
        {
            search {
              query(query: \"bar\") {
                edges {
                  node {
                    __typename
                    ... on Dataset {
                      id
                    }
                  }
                }
              }
            }
          }
        ",
        )
        .await;
    assert!(res.is_ok());
    assert_eq!(
        res.data,
        value!({
            "search": {
                "query": {
                    "edges": [],
                }
            }
        })
    );

    let res = schema
        .execute(
            "
        {
            search {
              query(query: \"foo\") {
                edges {
                  node {
                    __typename
                    ... on Dataset {
                      id
                    }
                  }
                }
              }
            }
          }
        ",
        )
        .await;
    assert!(res.is_ok());
    assert_eq!(
        res.data,
        value!({
            "search": {
                "query": {
                    "edges": [{
                        "node": {
                            "__typename": "Dataset",
                            "id": "foo",
                        }
                    }],
                }
            }
        })
    );
}
