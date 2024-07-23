// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use prometheus::Encoder as _;

#[allow(clippy::unused_async)]
pub async fn metrics_handler(
    axum::extract::Extension(reg): axum::extract::Extension<prometheus::Registry>,
) -> String {
    let mut buf = Vec::new();

    prometheus::TextEncoder::new()
        .encode(&reg.gather(), &mut buf)
        .unwrap();

    String::from_utf8(buf).unwrap()
}
