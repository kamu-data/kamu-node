// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use http::Uri;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn http_layer() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    MakeSpan,
    OnRequest,
    OnResponse,
> {
    tower_http::trace::TraceLayer::new_for_http()
        .on_request(OnRequest)
        .on_response(OnResponse)
        .make_span_with(MakeSpan)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct OnRequest;

impl<B> tower_http::trace::OnRequest<B> for OnRequest {
    fn on_request(&mut self, request: &http::Request<B>, _: &tracing::Span) {
        tracing::info!(
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers(),
            "HTTP request",
        );
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct OnResponse;

impl<B> tower_http::trace::OnResponse<B> for OnResponse {
    fn on_response(
        self,
        response: &http::Response<B>,
        latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        tracing::info!(
            status = response.status().as_u16(),
            headers = ?response.headers(),
            latency = %Latency(latency),
            "HTTP response"
        );
    }
}

struct Latency(std::time::Duration);

impl std::fmt::Display for Latency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ms", self.0.as_millis())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for MakeSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        use opentelemetry::trace::TraceContextExt as _;
        use tracing_opentelemetry::OpenTelemetrySpanExt as _;

        // TODO: Extract parent context from the request

        let method = request.method();
        let route = RouteOrUri::from(request);

        let span = tracing::info_span!(
            "http_request",
            %method,
            %route,
            // Placeholders for OTEL fileds that will be populated after span creation
            trace_id = tracing::field::Empty,
            "otel.name" = tracing::field::Empty,
        );

        // Extract trace ID from the OTEL context and add it to the tracing span
        let context = span.context();
        let otel_span = context.span();
        let span_context = otel_span.span_context();
        let trace_id = span_context.trace_id();
        if span_context.is_valid() {
            span.record("trace_id", tracing::field::display(trace_id));
            span.record(
                "otel.name",
                tracing::field::display(SpanName::new(method, route)),
            );
        }

        span
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct SpanName<'a> {
    method: &'a http::Method,
    route: RouteOrUri<'a>,
}

impl<'a> SpanName<'a> {
    fn new(method: &'a http::Method, route: RouteOrUri<'a>) -> Self {
        Self { method, route }
    }
}

impl<'a> std::fmt::Display for SpanName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method, self.route)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

enum RouteOrUri<'a> {
    Route(&'a str),
    Uri(&'a Uri),
}

impl<'a, B> From<&'a http::Request<B>> for RouteOrUri<'a> {
    fn from(request: &'a http::Request<B>) -> Self {
        request
            .extensions()
            .get::<axum::extract::MatchedPath>()
            .map_or_else(
                || RouteOrUri::Uri(request.uri()),
                |m| RouteOrUri::Route(m.as_str()),
            )
    }
}

impl<'a> std::fmt::Display for RouteOrUri<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteOrUri::Route(s) => write!(f, "{s}"),
            RouteOrUri::Uri(uri) => write!(f, "{uri}"),
        }
    }
}
