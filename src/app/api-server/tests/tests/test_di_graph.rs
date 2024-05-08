// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

#[test_log::test(tokio::test)]
async fn test_di_graph_validates_local() {
    let tempdir = tempfile::tempdir().unwrap();

    let multi_tenant = false;
    let config = kamu_api_server::config::ApiServerConfig::default();
    let repo_url = url::Url::from_directory_path(tempdir.path()).unwrap();

    let dependencies_graph_repository = kamu_api_server::prepare_dependencies_graph_repository(
        kamu_accounts::CurrentAccountSubject::new_test(),
        &repo_url,
        multi_tenant,
        &config,
    )
    .await;

    let mut catalog_builder =
        kamu_api_server::init_dependencies(config, &repo_url, multi_tenant, tempdir.path()).await;

    catalog_builder.add_value(dependencies_graph_repository);

    // CurrentAccountSubject is inserted by middlewares, but won't be present in
    // the default dependency graph, so we have to add it manually
    catalog_builder.add_value(kamu_accounts::CurrentAccountSubject::new_test());

    // TODO: We should ensure this test covers parameters requested by commands and
    // types needed for GQL/HTTP adapter that are currently being constructed
    // manually
    let validate_result = catalog_builder.validate();

    assert!(
        validate_result.is_ok(),
        "{}",
        validate_result.err().unwrap()
    );
}

#[test_group::group(containerized)]
#[test_log::test(tokio::test)]
async fn test_di_graph_validates_remote() {
    let access_key = "AKIAIOSFODNN7EXAMPLE";
    let secret_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    std::env::set_var("AWS_ACCESS_KEY_ID", access_key);
    std::env::set_var("AWS_SECRET_ACCESS_KEY", secret_key);

    let tmp_repo_dir = tempfile::tempdir().unwrap();
    let bucket = "test-bucket";
    std::fs::create_dir(tmp_repo_dir.path().join(bucket)).unwrap();

    let minio = kamu::testing::MinioServer::new(tmp_repo_dir.path(), access_key, secret_key).await;

    use std::str::FromStr;
    let repo_url = url::Url::from_str(&format!(
        "s3+http://{}:{}/{}",
        minio.address, minio.host_port, bucket
    ))
    .unwrap();

    let multi_tenant = true;
    let config = kamu_api_server::config::ApiServerConfig::default();

    let dependencies_graph_repository = kamu_api_server::prepare_dependencies_graph_repository(
        kamu_accounts::CurrentAccountSubject::new_test(),
        &repo_url,
        multi_tenant,
        &config,
    )
    .await;

    let mut catalog_builder = kamu_api_server::init_dependencies(
        kamu_api_server::config::ApiServerConfig::default(),
        &repo_url,
        multi_tenant,
        tmp_repo_dir.path(),
    )
    .await;

    catalog_builder.add_value(dependencies_graph_repository);

    // CurrentAccountSubject is inserted by middlewares, but won't be present in
    // the default dependency graph, so we have to add it manually
    catalog_builder.add_value(kamu_accounts::CurrentAccountSubject::new_test());

    // TODO: We should ensure this test covers parameters requested by commands and
    // types needed for GQL/HTTP adapter that are currently being constructed
    // manually
    let validate_result = catalog_builder.validate();

    assert!(
        validate_result.is_ok(),
        "{}",
        validate_result.err().unwrap()
    );
}
