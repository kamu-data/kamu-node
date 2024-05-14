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
use datafusion::prelude::SessionContext;
use futures::Future;
use kamu_accounts::{AnonymousAccountReason, CurrentAccountSubject};
use kamu_adapter_flight_sql::{KamuFlightSqlService, SessionFactory, Token};
use tokio::net::TcpListener;
use tonic::transport::Server;
use tonic::Status;

use crate::config::ACCOUNT_KAMU;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct FlightSqlServer {
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
            .with_session_factory(Arc::new(SessionFactoryImpl { base_catalog }))
            .build();

        let listener = TcpListener::bind((address, port.unwrap_or_default()))
            .await
            .unwrap();

        Self { service, listener }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub fn run(self) -> impl Future<Output = Result<(), impl std::error::Error>> {
        Server::builder()
            .add_service(FlightServiceServer::new(self.service))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                self.listener,
            ))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

struct SessionFactoryImpl {
    base_catalog: dill::Catalog,
}

#[async_trait::async_trait]
impl SessionFactory for SessionFactoryImpl {
    #[tracing::instrument(level = "debug", skip_all, fields(username))]
    async fn authenticate(&self, username: &str, password: &str) -> Result<Token, Status> {
        // TODO: SEC: Real auth via app token
        if username == ACCOUNT_KAMU && password == username {
            Ok(String::new())
        } else {
            Err(Status::unauthenticated("Invalid credentials!"))
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_context(&self, _token: &Token) -> Result<Arc<SessionContext>, Status> {
        let subject =
            CurrentAccountSubject::Anonymous(AnonymousAccountReason::NoAuthenticationProvided);

        let session_catalog = dill::CatalogBuilder::new_chained(&self.base_catalog)
            .add_value(subject)
            .build();

        let query_svc = session_catalog
            .get_one::<dyn kamu::domain::QueryService>()
            .unwrap();

        let ctx = Arc::new(query_svc.create_session().await.unwrap());

        Ok(ctx)
    }
}
