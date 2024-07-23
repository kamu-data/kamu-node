// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

/// Returns a future that completes when SIGINT or SIGTERM signal is received.
/// Can be combined with facilities like Axum's [`with_graceful_shutdown`](https://docs.rs/axum/latest/axum/serve/struct.Serve.html#method.with_graceful_shutdown).
pub async fn trap_signals() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::warn!("SIGINT signal received, shutting down gracefully");
        },
        _ = terminate => {
            tracing::warn!("SIGTERM signal received, shutting down gracefully");
        },
    }
}
