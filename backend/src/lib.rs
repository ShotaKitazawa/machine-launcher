use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::{http::StatusCode, response::Response};
use openidconnect::core::CoreClient;
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet};

use crate::drivers::traits::{PowerManagerTrait, PowerStatus};

pub type OidcClient<HasTokenUrl = EndpointMaybeSet, HasUserInfoUrl = EndpointMaybeSet> = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    HasTokenUrl,
    HasUserInfoUrl,
>;

pub enum OidcState {
    Disabled,
    Enabled {
        client: Box<OidcClient>,
        client_id: String,
        authorization_endpoint: String,
        token_endpoint: String,
        allowed_subs: HashSet<String>,
    },
}

pub struct AppState {
    pub drivers: HashMap<String, Arc<dyn PowerManagerTrait>>,
    pub oidc: OidcState,
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
        tracing::error!("{}", self);
        (
            self.status_code(),
            axum::Json(machine_launcher_common::ErrorMessage {
                error: format!("{}", self),
            }),
        )
            .into_response()
    }
}

pub mod cmd;
pub mod drivers;
pub mod handlers_app;
pub mod handlers_config;
pub mod middlewares;

#[cfg(feature = "openapi-gen")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        handlers_app::machine_status,
        handlers_app::start_machine,
        handlers_app::stop_machine,
        handlers_config::oidc_config,
    ),
    components(schemas(
        machine_launcher_common::Server,
        machine_launcher_common::ServerName,
        machine_launcher_common::ErrorMessage,
        machine_launcher_common::OidcConfigResponse,
    ))
)]
pub struct ApiDoc;
