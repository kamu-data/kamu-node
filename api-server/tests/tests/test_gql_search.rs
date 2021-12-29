use async_graphql::*;

use kamu::domain::*;
use kamu::infra;
use kamu::testing::MetadataFactory;

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
                .name("foo")
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
                nodes {
                  __typename
                  ... on Dataset {
                    name
                  }
                }
                totalCount
                pageInfo {
                  totalPages
                  hasNextPage
                  hasPreviousPage
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
                    "nodes": [],
                    "totalCount": 0,
                    "pageInfo": {
                        "totalPages": 0,
                        "hasNextPage": false,
                        "hasPreviousPage": false,
                    }
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
                nodes {
                  __typename
                  ... on Dataset {
                    name
                  }
                }
                totalCount
                pageInfo {
                  totalPages
                  hasNextPage
                  hasPreviousPage
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
                    "nodes": [{
                        "__typename": "Dataset",
                        "name": "foo",
                    }],
                    "totalCount": 1,
                    "pageInfo": {
                        "totalPages": 1,
                        "hasNextPage": false,
                        "hasPreviousPage": false,
                    }
                }
            }
        })
    );
}
