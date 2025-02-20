// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use indoc::indoc;
use kamu_cli_e2e_common::KamuApiServerClientExt;
use kamu_node_e2e_common::KamuFlightSQLClient;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_flight_sql_self_test(mut kamu_flight_sql_client: KamuFlightSQLClient) {
    kamu_flight_sql_client.set_anonymous().await;

    kamu_flight_sql_client
        .flight_sql_assert_call(
            "select 1 as value",
            Some(indoc!(
                "
        message arrow_schema {
          REQUIRED INT64 value;
        }
        "
            )),
            None,
        )
        .await;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_flight_sql_anonymous(mut kamu_flight_sql_client: KamuFlightSQLClient) {
    kamu_flight_sql_client
        .kamu_api_client()
        .auth()
        .login_as_kamu()
        .await;
    kamu_flight_sql_client
        .kamu_api_client()
        .dataset()
        .create_player_scores_dataset_with_data()
        .await;

    kamu_flight_sql_client.set_anonymous().await;

    kamu_flight_sql_client
        .flight_sql_assert_call(
            "select op, match_time, match_id, player_id, score from 'kamu/player-scores'",
            Some(indoc!(
                "
        message arrow_schema {
          REQUIRED INT32 op;
          OPTIONAL INT64 match_time (TIMESTAMP(MILLIS,true));
          OPTIONAL INT64 match_id;
          OPTIONAL BYTE_ARRAY player_id (STRING);
          OPTIONAL INT64 score;
        }
        "
            )),
            Some(indoc!(
                "
                +----+----------------------+----------+-----------+-------+
                | op | match_time           | match_id | player_id | score |
                +----+----------------------+----------+-----------+-------+
                | 0  | 2000-01-01T00:00:00Z | 1        | Alice     | 100   |
                | 0  | 2000-01-01T00:00:00Z | 1        | Bob       | 80    |
                +----+----------------------+----------+-----------+-------+
                "
            )),
        )
        .await;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_flight_sql(mut kamu_flight_sql_client: KamuFlightSQLClient) {
    let token = kamu_flight_sql_client
        .kamu_api_client()
        .auth()
        .login_as_kamu()
        .await;
    kamu_flight_sql_client
        .kamu_api_client()
        .dataset()
        .create_player_scores_dataset_with_data()
        .await;

    kamu_flight_sql_client.set_token(token);

    kamu_flight_sql_client
        .flight_sql_assert_call(
            "select op, match_time, match_id, player_id, score from 'kamu/player-scores'",
            Some(indoc!(
                "
        message arrow_schema {
          REQUIRED INT32 op;
          OPTIONAL INT64 match_time (TIMESTAMP(MILLIS,true));
          OPTIONAL INT64 match_id;
          OPTIONAL BYTE_ARRAY player_id (STRING);
          OPTIONAL INT64 score;
        }
        "
            )),
            Some(indoc!(
                "
                +----+----------------------+----------+-----------+-------+
                | op | match_time           | match_id | player_id | score |
                +----+----------------------+----------+-----------+-------+
                | 0  | 2000-01-01T00:00:00Z | 1        | Alice     | 100   |
                | 0  | 2000-01-01T00:00:00Z | 1        | Bob       | 80    |
                +----+----------------------+----------+-----------+-------+
                "
            )),
        )
        .await;
}
