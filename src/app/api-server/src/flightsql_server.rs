// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

/////////////////////////////////////////////////////////////////////////////////////////

use std::net::SocketAddr;
use std::sync::Arc;

use arrow_flight::flight_service_server::FlightServiceServer;
use datafusion::logical_expr::LogicalPlan;
use datafusion::prelude::SessionContext;
use futures::Future;
use kamu_accounts::{AnonymousAccountReason, CurrentAccountSubject};
use kamu_adapter_flight_sql::{
    KamuFlightSqlService,
    PlanToken,
    SessionManager,
    SessionManagerCaching,
    SessionToken,
};
use tokio::net::TcpListener;
use tonic::transport::Server;
use tonic::Status;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct FlightSqlServer {
    base_catalog: dill::Catalog,
    service: KamuFlightSqlService,
    listener: TcpListener,
}

/////////////////////////////////////////////////////////////////////////////////////////

impl FlightSqlServer {
    pub async fn new(
        address: std::net::IpAddr,
        port: Option<u16>,
        base_catalog: dill::Catalog,
    ) -> Self {
        let service = kamu_adapter_flight_sql::KamuFlightSqlService::builder()
            .with_server_name(crate::BINARY_NAME, crate::VERSION)
            .build();

        let listener = TcpListener::bind((address, port.unwrap_or_default()))
            .await
            .unwrap();

        Self {
            base_catalog,
            service,
            listener,
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub fn run(self) -> impl Future<Output = Result<(), impl std::error::Error>> {
        Server::builder()
            .trace_fn(trace_grpc_request)
            .layer(tonic::service::interceptor(
                move |mut req: tonic::Request<()>| {
                    req.extensions_mut().insert(self.base_catalog.clone());
                    Ok(req)
                },
            ))
            .add_service(FlightServiceServer::new(self.service))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                self.listener,
            ))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Move to `kamu-adapter-flightsql`
fn trace_grpc_request(request: &http::Request<()>) -> tracing::Span {
    let (service, method) = request
        .uri()
        .path()
        .strip_prefix('/')
        .and_then(|s| s.split_once('/'))
        .unzip();

    let otel_name = request.uri().path().strip_prefix('/');

    observability::tracing::root_span!(
        "flightsql_request",
        service = service.unwrap_or_default(),
        method = method.unwrap_or_default(),
        "otel.name" = otel_name.unwrap_or_default(),
    )
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Transactional wrapper on top of [`SessionManagerCaching`]
#[dill::component]
#[dill::interface(dyn SessionManager)]
pub struct SessionManagerCachingTransactional {
    base_catalog: dill::Catalog,
}

impl SessionManagerCachingTransactional {
    async fn inner(&self) -> Result<Arc<SessionManagerCaching>, Status> {
        let subject =
            CurrentAccountSubject::Anonymous(AnonymousAccountReason::NoAuthenticationProvided);

        // Extract transaction manager, specific for the database
        let db_transaction_manager = self
            .base_catalog
            .get_one::<dyn database_common::DatabaseTransactionManager>()
            .unwrap();

        // This is a read-only transaction, so we don't need a COMMIT.
        // It will be automatically rolled back when catalog is dropped.
        let transaction_ref = db_transaction_manager.make_transaction_ref().await.map_err(|e| {
            tracing::error!(error = %e, error_dbg = ?e, "Failed to open database transaction for FlightSQL session");
            Status::internal("could not start database transaction")
        })?;

        let session_catalog = dill::CatalogBuilder::new_chained(&self.base_catalog)
            .add_value(subject)
            .add_value(transaction_ref)
            .build();

        session_catalog
            .get_one()
            .map_err(|e| Status::internal(e.to_string()))
    }
}

#[async_trait::async_trait]
impl SessionManager for SessionManagerCachingTransactional {
    async fn auth_basic(&self, username: &str, password: &str) -> Result<SessionToken, Status> {
        self.inner().await?.auth_basic(username, password).await
    }

    async fn end_session(&self, token: &SessionToken) -> Result<(), Status> {
        self.inner().await?.end_session(token).await
    }

    async fn get_context(&self, token: &SessionToken) -> Result<Arc<SessionContext>, Status> {
        self.inner().await?.get_context(token).await
    }

    async fn cache_plan(
        &self,
        token: &SessionToken,
        plan: LogicalPlan,
    ) -> Result<PlanToken, Status> {
        self.inner().await?.cache_plan(token, plan).await
    }

    async fn get_plan(
        &self,
        token: &SessionToken,
        plan_token: &PlanToken,
    ) -> Result<LogicalPlan, Status> {
        self.inner().await?.get_plan(token, plan_token).await
    }

    async fn remove_plan(
        &self,
        token: &SessionToken,
        plan_token: &PlanToken,
    ) -> Result<(), Status> {
        self.inner().await?.remove_plan(token, plan_token).await
    }
}
