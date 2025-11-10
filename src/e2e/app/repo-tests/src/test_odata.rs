// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use kamu_cli_e2e_common::{KamuApiServerClient, KamuApiServerClientExt};
use reqwest::Method;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_odata_service_handler(mut kamu_api_server_client: KamuApiServerClient) {
    kamu_api_server_client.auth().login_as_kamu().await;
    kamu_api_server_client
        .dataset()
        .create_player_scores_dataset()
        .await;

    let res = kamu_api_server_client
        .rest_api_call(Method::GET, "/odata/kamu", None)
        .await;

    pretty_assertions::assert_eq!(
        res.text().await.unwrap(),
        indoc::indoc!(
            r#"
        <?xml version="1.0" encoding="utf-8"?>
        <service xml:base="<base_url>odata"
         xmlns="http://www.w3.org/2007/app"
         xmlns:atom="http://www.w3.org/2005/Atom">
        <workspace>
        <atom:title>default</atom:title>
        <collection href="player-scores">
        <atom:title>player-scores</atom:title>
        </collection>
        </workspace>
        </service>
        "#
        )
        .replace('\n', "")
        .replace("<base_url>", kamu_api_server_client.get_base_url().as_str())
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_odata_metadata_handler(mut kamu_api_server_client: KamuApiServerClient) {
    kamu_api_server_client.auth().login_as_kamu().await;
    kamu_api_server_client
        .dataset()
        .create_player_scores_dataset_with_data()
        .await;

    let res = kamu_api_server_client
        .rest_api_call(Method::GET, "/odata/kamu/$metadata", None)
        .await;

    pretty_assertions::assert_eq!(
        res.text().await.unwrap(),
        indoc::indoc!(
            r#"
        <?xml version="1.0" encoding="utf-8"?>
        <edmx:Edmx xmlns:edmx="http://schemas.microsoft.com/ado/2007/06/edmx" Version="1.0">
        <edmx:DataServices xmlns:m="http://schemas.microsoft.com/ado/2007/08/dataservices/metadata"
         m:DataServiceVersion="3.0"
         m:MaxDataServiceVersion="3.0"
        >
        <Schema Namespace="default" xmlns="http://schemas.microsoft.com/ado/2009/11/edm">
        <EntityType Name="player-scores">
        <Key>
        <PropertyRef Name="offset"/>
        </Key>
        <Property Name="offset" Type="Edm.Int64" Nullable="false"/>
        <Property Name="op" Type="Edm.Int32" Nullable="false"/>
        <Property Name="system_time" Type="Edm.DateTimeOffset" Nullable="false"/>
        <Property Name="match_time" Type="Edm.DateTimeOffset" Nullable="false"/>
        <Property Name="match_id" Type="Edm.Int64" Nullable="true"/>
        <Property Name="player_id" Type="Edm.String" Nullable="true"/>
        <Property Name="score" Type="Edm.Int64" Nullable="true"/>
        </EntityType>
        <EntityContainer Name="default" m:IsDefaultEntityContainer="true">
        <EntitySet Name="player-scores" EntityType="default.player-scores"/>
        </EntityContainer>
        </Schema>
        </edmx:DataServices>
        </edmx:Edmx>
        "#
        )
        .replace('\n', "")
    );

    // Make sure we correctly cover trailing slashes
    let res = kamu_api_server_client
        .rest_api_call(Method::GET, "/odata/kamu/$metadata/", None)
        .await;

    pretty_assertions::assert_eq!(
            res.text().await.unwrap(),
            indoc::indoc!(
                r#"
            <?xml version="1.0" encoding="utf-8"?>
            <edmx:Edmx xmlns:edmx="http://schemas.microsoft.com/ado/2007/06/edmx" Version="1.0">
            <edmx:DataServices xmlns:m="http://schemas.microsoft.com/ado/2007/08/dataservices/metadata"
             m:DataServiceVersion="3.0"
             m:MaxDataServiceVersion="3.0"
            >
            <Schema Namespace="default" xmlns="http://schemas.microsoft.com/ado/2009/11/edm">
            <EntityType Name="player-scores">
            <Key>
            <PropertyRef Name="offset"/>
            </Key>
            <Property Name="offset" Type="Edm.Int64" Nullable="false"/>
            <Property Name="op" Type="Edm.Int32" Nullable="false"/>
            <Property Name="system_time" Type="Edm.DateTimeOffset" Nullable="false"/>
            <Property Name="match_time" Type="Edm.DateTimeOffset" Nullable="false"/>
            <Property Name="match_id" Type="Edm.Int64" Nullable="true"/>
            <Property Name="player_id" Type="Edm.String" Nullable="true"/>
            <Property Name="score" Type="Edm.Int64" Nullable="true"/>
            </EntityType>
            <EntityContainer Name="default" m:IsDefaultEntityContainer="true">
            <EntitySet Name="player-scores" EntityType="default.player-scores"/>
            </EntityContainer>
            </Schema>
            </edmx:DataServices>
            </edmx:Edmx>
            "#
            )
            .replace('\n', "")
        );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_odata_collection_handler(mut kamu_api_server_client: KamuApiServerClient) {
    kamu_api_server_client.auth().login_as_kamu().await;
    kamu_api_server_client
        .dataset()
        .create_player_scores_dataset_with_data()
        .await;

    let res = kamu_api_server_client
        .rest_api_call(Method::GET, "/odata/kamu/player-scores", None)
        .await;

    let res_text = res.text().await.unwrap();
    // ToDo Such as odata contains DateTime fields we make result checks in this
    // ugly way We can select only specific fields via query params
    // $select=match_id,player_id,score but odata still returns `<updated>`
    // field for each entity which corellates with system_time
    assert!(res_text.contains("<d:match_id m:type=\"Edm.Int64\">1</d:match_id>"));
    assert!(res_text.contains("<d:player_id m:type=\"Edm.String\">Alice</d:player_id>"));
    assert!(res_text.contains("<d:score m:type=\"Edm.Int64\">100</d:score>"));
    assert!(res_text.contains("<d:player_id m:type=\"Edm.String\">Bob</d:player_id>"));
    assert!(res_text.contains("<d:score m:type=\"Edm.Int64\">80</d:score>"));

    // Make sure we correctly cover trailing slashes
    let res = kamu_api_server_client
        .rest_api_call(Method::GET, "/odata/kamu/player-scores/", None)
        .await;

    let res_text = res.text().await.unwrap();

    assert!(res_text.contains("<d:match_id m:type=\"Edm.Int64\">1</d:match_id>"));
    assert!(res_text.contains("<d:player_id m:type=\"Edm.String\">Alice</d:player_id>"));
    assert!(res_text.contains("<d:score m:type=\"Edm.Int64\">100</d:score>"));
    assert!(res_text.contains("<d:player_id m:type=\"Edm.String\">Bob</d:player_id>"));
    assert!(res_text.contains("<d:score m:type=\"Edm.Int64\">80</d:score>"));
}
