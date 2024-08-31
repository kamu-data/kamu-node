// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

/////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    let matches = kamu_api_server::cli().get_matches();

    let config = kamu_api_server::load_config(matches.get_one("config")).unwrap();

    let mut builder = tokio::runtime::Builder::new_multi_thread();

    if let Some(worker_threads) = config.runtime.worker_threads {
        builder.worker_threads(worker_threads);
    }
    if let Some(max_blocking_threads) = config.runtime.max_blocking_threads {
        builder.max_blocking_threads(max_blocking_threads);
    }
    if let Some(thread_stack_size) = config.runtime.thread_stack_size {
        builder.thread_stack_size(thread_stack_size);
    }

    let rt = builder.enable_all().build().unwrap();

    let code = rt.block_on(main_async(matches, config));

    std::process::exit(code)
}

async fn main_async(
    args: clap::ArgMatches,
    config: kamu_api_server::config::ApiServerConfig,
) -> i32 {
    use kamu_api_server::app;

    let _guard = app::init_observability();

    match kamu_api_server::app::run(args, config).await {
        Ok(_) => 0,
        Err(err) => {
            tracing::error!(error = %err, error_dbg = ?err, "Server exited with error");
            1
        }
    }
}
