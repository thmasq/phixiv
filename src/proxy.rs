use std::env;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{middleware, Router};
use http::header::{HeaderValue, CONTENT_TYPE};
use tokio::sync::RwLock;

use crate::helper::{self, PhixivError};
use crate::state::{authorized_middleware, PhixivState};

async fn proxy_handler(
	State(state): State<Arc<RwLock<PhixivState>>>,
	Path(path): Path<String>,
) -> Result<impl IntoResponse, PhixivError> {
	let base = env::var("PXIMG_BASE").unwrap_or_else(|_| String::from("https://i.pximg.net/"));
	let url = format!("{base}{path}");

	let mut headers = helper::headers();
	headers.append("Referer", "https://www.pixiv.net/".parse()?);

	let response = state.read().await.client.get(&url).headers(headers).send().await?;

	let status = response.status();
	let content_type = response.headers().get(CONTENT_TYPE).cloned();
	let bytes = response.bytes().await?;
	let mut res = Response::new(Body::from(bytes));
	*res.status_mut() = status;

	if let Some(content_type) = content_type {
		res.headers_mut().insert(CONTENT_TYPE, content_type);
	} else {
		res.headers_mut()
			.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
	}

	Ok(res)
}

#[allow(clippy::module_name_repetitions)]
pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
	Router::new()
		.route("/*path", get(proxy_handler))
		.layer(middleware::from_fn_with_state(
			state,
			authorized_middleware::<axum::body::Body>,
		))
}
