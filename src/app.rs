use axum::{routing::get, Router};

use crate::http::credential::get_credentials_handler;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) sts_client: aws_sdk_sts::Client,
    pub(crate) auth_token: String,
    pub(crate) role_arn: String,
    pub(crate) role_session_name: String,
}

impl AppState {
    pub fn new(
        sts_client: aws_sdk_sts::Client,
        auth_token: String,
        role_arn: String,
        role_session_name: String,
    ) -> Self {
        Self {
            sts_client,
            auth_token,
            role_arn,
            role_session_name,
        }
    }
}

pub(crate) fn assumer_app(state: AppState) -> Router {
    Router::new()
        .route("/get-credentials", get(get_credentials_handler))
        .with_state(state)
}
