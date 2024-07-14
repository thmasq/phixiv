pub mod api;
pub mod auth;
pub mod embed;
pub mod helper;
pub mod oembed;
pub mod pixiv;
pub mod proxy;
pub mod state;

use std::{env, net::SocketAddr, sync::Arc};

use api::api_router;
use axum::{response::IntoResponse, routing::get, Json, Router};
use oembed::oembed_handler;
use proxy::proxy_router;
use serde_json::json;
use state::PhixivState;
use tokio::sync::RwLock;
use tower_http::{
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let addr: SocketAddr = format!(
        "[::]:{}",
        env::var("PORT").unwrap_or_else(|_| String::from("3000"))
    )
    .parse()?;

    let tracing_registry = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env());

    tracing_registry.init();

    tracing::info!("Listening on: {addr}");

    let state = Arc::new(RwLock::new(
        PhixivState::login(env::var("PIXIV_REFRESH_TOKEN")?).await?,
    ));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app(state).into_make_service()).await?;

    Ok(())
}

fn app(state: Arc<RwLock<PhixivState>>) -> Router {
    Router::new()
        .merge(embed::router(state.clone()))
        .route("/health", get(health))
        .route("/e", get(oembed_handler))
        .nest("/i", proxy_router(state.clone()))
        .nest("/api", api_router(state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(NormalizePathLayer::trim_trailing_slash())
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(json!({ "health": "UP" }))
}
