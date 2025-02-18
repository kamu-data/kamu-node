// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use indoc::indoc;
use kamu_node_e2e_common::KamuFlightSQLClient;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn test_flight_sql_anonymous(mut kamu_flight_sql_client: KamuFlightSQLClient) {
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
        )
        .await;
}
