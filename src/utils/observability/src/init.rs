// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::io::IsTerminal;
use std::time::Duration;

use opentelemetry_otlp::WithExportConfig as _;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::EnvFilter;

use super::config::Config;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// If application is started under a terminal will use the [`dev`] mode,
/// otherwise will use [`service`] mode.
pub fn auto(cfg: Config) -> Guard {
    if std::io::stderr().is_terminal() {
        dev(cfg)
    } else {
        service(cfg)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::needless_pass_by_value)]
pub fn dev(cfg: Config) -> Guard {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(cfg.default_log_levels.clone()));

    let text_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(std::io::stderr)
        .with_line_number(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    let (otel_layer, otlp_guard) = if cfg.otlp_endpoint.is_none() {
        (None, None)
    } else {
        (
            Some(
                tracing_opentelemetry::layer()
                    .with_error_records_to_exceptions(true)
                    .with_tracer(init_otel_tracer(&cfg)),
            ),
            Some(OtlpGuard),
        )
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(otel_layer)
        .with(text_layer)
        .init();

    Guard {
        non_blocking_appender: None,
        otlp_guard,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::needless_pass_by_value)]
pub fn service(cfg: Config) -> Guard {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(cfg.default_log_levels.clone()));

    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());

    let text_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_line_number(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    let (otel_layer, otlp_guard) = if cfg.otlp_endpoint.is_none() {
        (None, None)
    } else {
        (
            Some(
                tracing_opentelemetry::layer()
                    .with_error_records_to_exceptions(true)
                    .with_tracer(init_otel_tracer(&cfg)),
            ),
            Some(OtlpGuard),
        )
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(otel_layer)
        .with(text_layer)
        .init();

    Guard {
        non_blocking_appender: Some(guard),
        otlp_guard,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn init_otel_tracer(cfg: &Config) -> opentelemetry_sdk::trace::Tracer {
    use opentelemetry::KeyValue;
    use opentelemetry_semantic_conventions::resource as otel_resource;

    let otel_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(cfg.otlp_endpoint.as_ref().unwrap())
        .with_timeout(Duration::from_secs(5));

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otel_exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_resource(opentelemetry_sdk::Resource::new([
                    KeyValue::new(otel_resource::SERVICE_NAME, cfg.service_name.clone()),
                    KeyValue::new(otel_resource::SERVICE_VERSION, cfg.service_version.clone()),
                ]))
                .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Creating tracer")
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[must_use]
#[allow(dead_code)]
#[derive(Default)]
pub struct Guard {
    pub non_blocking_appender: Option<tracing_appender::non_blocking::WorkerGuard>,
    pub otlp_guard: Option<OtlpGuard>,
}

pub struct OtlpGuard;

impl Drop for OtlpGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
