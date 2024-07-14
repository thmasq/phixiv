use std::sync::Arc;

use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use http::Request;
use reqwest::Client;
use tokio::sync::RwLock;

use crate::auth::PixivAuth;
use crate::helper::PhixivError;

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct PhixivState {
	pub auth: PixivAuth,
	pub client: Client,
}

impl PhixivState {
	#[allow(clippy::missing_errors_doc)]
	pub async fn login(refresh_token: String) -> anyhow::Result<Self> {
		let client = Client::new();

		let auth = PixivAuth::login(&client, refresh_token).await?;

		Ok(Self { auth, client })
	}

	async fn refresh(&mut self) -> anyhow::Result<()> {
		self.auth.refresh(&self.client).await
	}
}

#[allow(clippy::missing_errors_doc)]
pub async fn authorized_middleware<B>(
	State(state): State<Arc<RwLock<PhixivState>>>,
	request: Request<axum::body::Body>,
	next: Next,
) -> Result<Response<axum::body::Body>, PhixivError> {
	if state.read().await.auth.expired() {
		state.write().await.refresh().await?;
	}

	Ok(next.run(request).await)
}
