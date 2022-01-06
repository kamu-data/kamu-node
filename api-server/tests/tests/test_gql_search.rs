use async_graphql::*;

use kamu::domain::*;
use kamu::infra;
use kamu::testing::MetadataFactory;
use opendatafabric::*;

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
                .kind(DatasetKind::Root)
                .push_event(MetadataFactory::set_polling_source().build())
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
                    "totalCount": 0i32,
                    "pageInfo": {
                        "totalPages": 0i32,
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
                    "totalCount": 1i32,
                    "pageInfo": {
                        "totalPages": 1i32,
                        "hasNextPage": false,
                        "hasPreviousPage": false,
                    }
                }
            }
        })
    );
}
