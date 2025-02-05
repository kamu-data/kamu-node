// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::net::Ipv4Addr;
use std::path::PathBuf;

use async_trait::async_trait;

use crate::KamuNodePuppet;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait KamuNodePuppetExt {
    async fn start_api_server(self, e2e_data_file_path: PathBuf) -> ServerOutput;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl KamuNodePuppetExt for KamuNodePuppet {
    async fn start_api_server(self, e2e_data_file_path: PathBuf) -> ServerOutput {
        let host = Ipv4Addr::LOCALHOST.to_string();

        let assert = self
            .execute([
                "--multi-tenant",
                "--e2e-output-data-path",
                e2e_data_file_path.to_str().unwrap(),
                "run",
                "--address",
                host.as_str(),
            ])
            .await
            .success();

        let stdout = std::str::from_utf8(&assert.get_output().stdout)
            .unwrap()
            .to_owned();
        let stderr = std::str::from_utf8(&assert.get_output().stderr)
            .unwrap()
            .to_owned();

        ServerOutput { stdout, stderr }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerOutput {
    pub stdout: String,
    pub stderr: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
