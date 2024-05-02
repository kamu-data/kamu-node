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

const DEFAULT_RUST_LOG: &str = "RUST_LOG=debug,kamu=trace,hyper=info,h2=info";

fn main() {
    let args = Cli::parse();

    let config: Config = confique::Config::builder()
        .env()
        .file(&args.config)
        .load()
        .unwrap();

    configure_tracing();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    match rt.block_on(kamu_oracle_provider::app::run(args, config)) {
        Ok(_) => {}
        Err(err) => {
            tracing::error!(error = %err, error_dbg = ?err, "Provider exited with error");
            std::process::exit(1)
        }
    }
}

fn configure_tracing() {
    use tracing_log::LogTracer;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(DEFAULT_RUST_LOG));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_ansi(true),
        )
        .init();

    // Redirect all standard logging to tracing events
    LogTracer::init().expect("Failed to set LogTracer");
}
