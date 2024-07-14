use std::{env, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    response::Response,
    routing::get,
    Router,
};
use http::header::{HeaderValue, CONTENT_TYPE};
use tokio::sync::RwLock;

use crate::{
    helper::{self, PhixivError},
    state::{authorized_middleware, PhixivState},
};

async fn proxy_handler(
    State(state): State<Arc<RwLock<PhixivState>>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, PhixivError> {
    let state = state.read().await;

    let base = env::var("PXIMG_BASE").unwrap_or_else(|_| String::from("https://i.pximg.net/"));
    let url = format!("{base}{path}");

    let mut headers = helper::headers();
    headers.append("Referer", "https://www.pixiv.net/".parse()?);

    let response = state.client.get(&url).headers(headers).send().await?;

    // Extract status and headers before consuming `response`
    let status = response.status();
    let content_type = response.headers().get(CONTENT_TYPE).cloned();

    // Now consume `response` to get the bytes
    let bytes = response.bytes().await?;

    // Construct the response manually
    let mut res = Response::new(Body::from(bytes));
    *res.status_mut() = status;

    // Insert the Content-Type header if it was present in the original response
    if let Some(content_type) = content_type {
        res.headers_mut().insert(CONTENT_TYPE, content_type);
    } else {
        // Default Content-Type if not specified
        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
    }

    Ok(res)
}

pub fn proxy_router(state: Arc<RwLock<PhixivState>>) -> Router<Arc<RwLock<PhixivState>>> {
    Router::new()
        .route("/*path", get(proxy_handler))
        .layer(middleware::from_fn_with_state(
            state,
            authorized_middleware::<axum::body::Body>,
        ))
}
