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

use arrow_flight::flight_service_server::FlightServiceServer;
use futures::Future;
use kamu_adapter_flight_sql::{AuthenticationLayer, KamuFlightSqlServiceWrapper};
use tokio::net::TcpListener;
use tonic::transport::Server;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct FlightSqlServer {
    catalog: dill::Catalog,
    listener: TcpListener,
}

/////////////////////////////////////////////////////////////////////////////////////////

impl FlightSqlServer {
    pub async fn new(address: std::net::IpAddr, port: Option<u16>, catalog: dill::Catalog) -> Self {
        let listener = TcpListener::bind((address, port.unwrap_or_default()))
            .await
            .unwrap();

        Self { catalog, listener }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub fn run(self) -> impl Future<Output = Result<(), impl std::error::Error>> {
        Server::builder()
            .layer(observability::tonic::grpc_layer())
            .layer(tonic::service::interceptor(
                move |mut req: tonic::Request<()>| {
                    req.extensions_mut().insert(self.catalog.clone());
                    Ok(req)
                },
            ))
            .layer(AuthenticationLayer::new())
            .add_service(FlightServiceServer::new(KamuFlightSqlServiceWrapper))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                self.listener,
            ))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
