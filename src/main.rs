pub mod api;
pub mod auth;
pub mod embed;
pub mod helper;
pub mod oembed;
pub mod pixiv;
pub mod proxy;
pub mod state;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use api::api_router;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use oembed::oembed_handler;
use proxy::proxy_router;
use serde_json::json;
use state::PhixivState;
use tokio::sync::RwLock;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();

	let addr: SocketAddr = format!(
		"127.0.0.1:{}",
		env::var("PORT").unwrap_or_else(|_| String::from("3000"))
	)
	.parse()?;

	let tracing_registry = tracing_subscriber::registry()
		.with(fmt::layer())
		.with(EnvFilter::from_default_env());

	tracing_registry.init();

	tracing::info!("Listening on: {addr}");

	let state = Arc::new(RwLock::new(PhixivState::login(env::var("PIXIV_REFRESH_TOKEN")?).await?));

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
