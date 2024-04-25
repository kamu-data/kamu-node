// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use clap::Parser;
use kamu_oracle_executor::{Cli, Config};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    let args = Cli::parse();

    let config: Config = confique::Config::builder()
        .env()
        .file(&args.config)
        .load()
        .unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    match rt.block_on(kamu_oracle_executor::run(args, config)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {err}\nDetails: {err:#?}");
            std::process::exit(1)
        }
    }
}
