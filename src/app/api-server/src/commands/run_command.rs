// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::{Future, IntoFuture as _};
use std::path::PathBuf;
use std::pin::Pin;

use internal_error::*;
use kamu::domain::TenancyConfig;
use kamu_accounts::CurrentAccountSubject;

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
    server_account_subject: CurrentAccountSubject,

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
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
impl Command for RunCommand {
    async fn run(&self) -> Result<(), InternalError> {
        let shutdown_requested = graceful_shutdown::trap_signals();

        // System services are built from the special catalog that contains the admin
        // subject. Thus all services that require authorization are granted full access
        // to all resources.
        //
        // TODO: Granting admin access to all system services is a security threat. We
        // should consider to instead propagate the auth info of the user who triggered
        // some system flow alongside all actions to enforce proper authorization.
        let system_catalog = self
            .catalog
            .builder_chained()
            .add_value(self.server_account_subject.clone())
            .build();

        init_on_startup::run_startup_jobs(&system_catalog)
            .await
            .int_err()?;

        let address = self
            .address
            .unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1).into());

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

        let task_agent = system_catalog
            .get_one::<dyn kamu_task_system::TaskAgent>()
            .unwrap();

        let flow_agent = system_catalog
            .get_one::<dyn kamu_flow_system::FlowAgent>()
            .unwrap();

        let outbox_agent = system_catalog
            .get_one::<messaging_outbox::OutboxAgent>()
            .unwrap();

        tracing::info!(
            http_endpoint = format!("http://{}", local_addr),
            flightsql_endpoint = format!("flightsql://{}", flightsql_server.local_addr()),
            "Serving traffic"
        );

        // TODO: Avoid using shutdown_notify in e2e and use signals instead
        let shutdown_future: Pin<Box<dyn Future<Output = ()> + Send>> =
            if let Some(shutdown_notify) = maybe_shutdown_notify {
                let combined = async move {
                    tokio::select! {
                        _ = shutdown_requested => {}
                        _ = shutdown_notify.notified() => {}
                    }
                };
                Box::pin(combined)
            } else {
                Box::pin(shutdown_requested)
            };

        let http_server = http_server
            .with_graceful_shutdown(shutdown_future)
            .into_future();

        // TODO: PERF: Do we need to spawn these into separate tasks?
        tokio::select! {
            res = http_server => { res.int_err() },
            res = flightsql_server.run() => { res.int_err() },
            res = task_agent.run() => { res.int_err() },
            res = flow_agent.run() => { res.int_err() },
            res = outbox_agent.run() => { res.int_err() },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
