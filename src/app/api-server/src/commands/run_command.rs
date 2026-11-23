// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::IntoFuture as _;
use std::path::PathBuf;

use background_agent_host::{BackgroundAgentHost, BackgroundAgentHostConfig, CancellationToken};
use internal_error::*;
use kamu::domain::{KamuBackgroundCatalog, TenancyConfig};

use super::{Command, CommandDesc};
use crate::ui_configuration::UIConfiguration;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[dill::component]
#[dill::interface(dyn Command)]
#[dill::meta(CommandDesc {
    needs_admin_auth: false,
    needs_transaction: false,
})]
pub struct RunCommand {
    catalog: dill::Catalog,

    tenancy_config: TenancyConfig,
    ui_config: UIConfiguration,

    #[dill::component(explicit)]
    address: Option<std::net::IpAddr>,
    #[dill::component(explicit)]
    http_port: Option<u16>,
    #[dill::component(explicit)]
    flightsql_port: Option<u16>,
    #[dill::component(explicit)]
    e2e_output_data_path: Option<PathBuf>,

    #[dill::component(explicit)]
    e2e_http_port: Option<u16>,

    #[dill::component(explicit)]
    read_only: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl Command for RunCommand {
    async fn run(&self) -> Result<(), InternalError> {
        let shutdown_requested = graceful_shutdown::trap_signals();

        // Startup jobs and background agents act for the system, so they are built from
        // the node catalog: it holds `Principal::Node` and no caller, so they pass every
        // permission check, but hold no account's signing key
        let system_catalog = self
            .catalog
            .get_one::<KamuBackgroundCatalog>()
            .int_err()?
            .node_catalog();

        if !self.read_only {
            init_on_startup::run_startup_jobs(&system_catalog)
                .await
                .int_err()?;
        }

        let address = self.address.unwrap_or(std::net::Ipv4Addr::LOCALHOST.into());

        // API servers are built from the regular catalog
        // that does not contain any auth subject, thus they will rely on
        // their own middlewares to authenticate per request / session and execute
        // all processing in the user context.
        let (http_server, local_addr, maybe_shutdown_notify) = crate::http_server::build_server(
            address,
            self.http_port,
            self.catalog.clone(),
            self.tenancy_config,
            self.ui_config.clone(),
            self.e2e_http_port,
            self.e2e_output_data_path.as_ref(),
        )
        .await?;

        let flightsql_server = crate::flightsql_server::FlightSqlServer::new(
            address,
            self.flightsql_port,
            self.catalog.clone(),
        )
        .await;

        if let Some(e2e_output_data_path) = &self.e2e_output_data_path {
            let e2e_file_content = format!(
                "http://{}\nhttp://{}",
                local_addr,
                flightsql_server.local_addr()
            );

            std::fs::write(e2e_output_data_path, e2e_file_content).unwrap();
        }

        let background_agent_host = system_catalog.get_one::<BackgroundAgentHost>().int_err()?;
        let shutdown_timeout = system_catalog
            .get_one::<BackgroundAgentHostConfig>()
            .int_err()?
            .shutdown_timeout;

        tracing::info!(
            http_endpoint = format!("http://{}", local_addr),
            flightsql_endpoint = format!("flightsql://{}", flightsql_server.local_addr()),
            "Serving traffic"
        );

        // One token stops both the servers and the agents
        let shutdown = CancellationToken::new();

        // TODO: Avoid using shutdown_notify in e2e and use signals instead
        let shutdown_requests = {
            let shutdown = shutdown.clone();
            async move {
                let e2e_shutdown = async {
                    match maybe_shutdown_notify {
                        Some(notify) => notify.notified().await,
                        None => std::future::pending().await,
                    }
                };
                tokio::select! {
                    () = shutdown.cancelled() => {}
                    () = shutdown_requested => {}
                    () = e2e_shutdown => {
                        tracing::info!("Shutdown requested through the e2e route");
                    }
                }
                shutdown.cancel();

                // From then on, a signal exits the process at once, in case the graceful
                // stop hangs. Detached, so that it does not hold up the end of a graceful stop
                tokio::spawn(async {
                    graceful_shutdown::trap_signals().await;
                    tracing::warn!("Second shutdown signal received, exiting at once");
                    std::process::exit(1);
                });
            }
        };

        let mut http_server = http_server
            .with_graceful_shutdown(shutdown.clone().cancelled_owned())
            .into_future();

        // On a shutdown, in-flight HTTP requests get the agents' shutdown timeout to
        // finish. Any server completing on its own stops everything else
        let servers = async {
            let result = tokio::select! {
                biased;

                () = shutdown.cancelled() => {
                    match tokio::time::timeout(shutdown_timeout, &mut http_server).await {
                        Ok(res) => res.int_err(),
                        Err(_) => {
                            tracing::warn!(
                                "HTTP grace period expired, remaining requests end with the process"
                            );
                            Ok(())
                        }
                    }
                }
                res = &mut http_server => {
                    tracing::warn!("HTTP server completed unexpectedly, shutting down");
                    res.int_err()
                }
                res = flightsql_server.run() => {
                    tracing::warn!("FlightSQL server completed unexpectedly, shutting down");
                    res.int_err()
                }
            };
            shutdown.cancel();
            result
        };

        if self.read_only {
            let (servers_result, ()) = tokio::join!(servers, shutdown_requests);
            servers_result
        } else {
            let (servers_result, agents_result, ()) = tokio::join!(
                servers,
                background_agent_host.run(shutdown.clone()),
                shutdown_requests,
            );
            agents_result?;
            servers_result
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
