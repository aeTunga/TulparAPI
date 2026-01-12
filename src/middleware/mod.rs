use axum::{body::Body, extract::ConnectInfo, http::{Method, Request}};
use governor::{
    clock::QuantaInstant,
    middleware::NoOpMiddleware,
};
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::KeyExtractor,
    GovernorLayer,
};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Level;

#[derive(Clone, Copy)]
pub struct SmartIpKeyExtractor;

impl KeyExtractor for SmartIpKeyExtractor {
    type Key = std::net::IpAddr;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, tower_governor::errors::GovernorError> {
        req.extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
            .or_else(|| {
                req.headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.split(',').next())
                    .and_then(|v| v.trim().parse().ok())
            })
            .ok_or(tower_governor::errors::GovernorError::UnableToExtractKey)
    }

    fn name(&self) -> &'static str {
        "SmartIpKeyExtractor"
    }
}

pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(3600))
}

pub fn trace() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&Request<Body>) -> tracing::Span + Clone,
> {
    TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        tracing::span!(
            Level::INFO,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = %request_id,
        )
    })
}

pub fn request_id() -> (SetRequestIdLayer<MakeRequestUuid>, PropagateRequestIdLayer) {
    (
        SetRequestIdLayer::x_request_id(MakeRequestUuid),
        PropagateRequestIdLayer::x_request_id(),
    )
}

pub fn rate_limit() -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(2)
        .burst_size(5)
        .finish()
        .expect("Failed to create governor config");
    
    GovernorLayer {
        config: Arc::new(governor_conf),
    }
}
