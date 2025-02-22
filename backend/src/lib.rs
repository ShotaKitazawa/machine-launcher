use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{http::StatusCode, response::Response};
use jmespath::Expression;
use openidconnect::core::CoreClient;
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet, PkceCodeVerifier};

use crate::drivers::traits::{PowerManagerTrait, PowerStatus};

pub type OidcClient<HasTokenUrl = EndpointMaybeSet, HasUserInfoUrl = EndpointMaybeSet> = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    HasTokenUrl,
    HasUserInfoUrl,
>;

pub struct AppState {
    pub drivers: HashMap<String, Arc<dyn PowerManagerTrait>>,
    pub oidc_audience: String,
    pub role_attribute_path_expr: Expression<'static>,
    pub pkce_verifiers: Mutex<HashMap<String, PkceCodeVerifier>>, // Store PKCE verifiers temporarily
    pub oidc_client: OidcClient<EndpointSet, EndpointMaybeSet>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    BadRequest(Cow<'static, str>),
    #[error("session error")]
    SessionError,
    #[error("unauthorized: {0}")]
    Unauthorized(Cow<'static, str>),
    #[error("forbidden: {0}")]
    Forbidden(Cow<'static, str>),
    #[error("not found: {0}")]
    NotFound(Cow<'static, str>),
    #[error("not implemented")]
    NotImplemented(),
    #[error("{0}")]
    InternalServerError(Cow<'static, str>),
}
impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) | Self::SessionError => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Io(_) | Self::NotImplemented() | Self::InternalServerError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> Response {
        #[derive(Debug, serde::Serialize)]
        struct ErrorResponse {
            error: String,
        }

        tracing::error!("{}", self);
        (
            self.status_code(),
            axum::Json(ErrorResponse {
                error: format!("{}", self),
            }),
        )
            .into_response()
    }
}

pub mod cmd;
pub mod drivers;
pub mod handlers_app;
pub mod handlers_oauth;
pub mod middlewares;
