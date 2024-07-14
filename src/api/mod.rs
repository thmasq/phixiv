mod info;

use std::sync::Arc;

use axum::routing::get;
use axum::{middleware, Router};
use tokio::sync::RwLock;

use crate::state::{authorized_middleware, PhixivState};

use self::info::artwork_info_handler;

#[allow(clippy::module_name_repetitions)]
pub fn api_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
	Router::new()
		.route("/info", get(artwork_info_handler))
		.layer(middleware::from_fn_with_state(
			state,
			authorized_middleware::<axum::body::Body>,
		))
}
