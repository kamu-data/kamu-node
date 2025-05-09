// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use clap::Parser;
use kamu_oracle_provider::{Cli, Config};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    // TODO: Currently we are compiling `rustls` with both `ring` and `aws-cl-rs`
    // backends and since v0.23 `rustls` requires to disambiguate between which
    // one to use. Eventually we should unify all dependencies around the same
    // backend, but a number of them don't yet expose the necessary feature flags.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default TLS provider");

    let args = Cli::parse();

    let config: Config = confique::Config::builder()
        .env()
        .file(&args.config)
        .load()
        .unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let code = rt.block_on(main_async(args, config));

    std::process::exit(code)
}

async fn main_async(args: Cli, config: Config) -> i32 {
    let _guard = kamu_oracle_provider::app::init_observability();
    match kamu_oracle_provider::app::run(args, config).await {
        Ok(_) => 0,
        Err(err) => {
            tracing::error!(error = %err, error_dbg = ?err, "Provider exited with error");
            1
        }
    }
}
