use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

use machine_launcher_common::{Endpoint, ListServers, Server, ServerName, StartServer, StopServer};
#[cfg(feature = "openapi-gen")]
use machine_launcher_common::ErrorMessage;

use crate::{AppState, Error};

pub fn routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(ListServers::PATH, get(machine_status))
        .route(StartServer::PATH, put(start_machine))
        .route(StopServer::PATH, put(stop_machine))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state,
            crate::middlewares::auth_middleware,
        ))
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    get,
    path = "/api/servers",
    responses(
        (status = 200, body = Vec<Server>),
        (status = 401, body = ErrorMessage),
        (status = 403, body = ErrorMessage),
    )
))]
async fn machine_status(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Server>>), Error> {
    let mut res: Vec<Server> = vec![];
    for driver in state.drivers.clone().values() {
        let status = driver.status()?;
        res.push(Server {
            name: status.name,
            hostname: status.hostname,
            running: status.running,
            reason: status.reason,
        });
    }
    Ok((StatusCode::OK, Json(res)))
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    put,
    path = "/api/servers/start",
    request_body = ServerName,
    responses(
        (status = 202, body = Server),
        (status = 400, body = ErrorMessage),
        (status = 401, body = ErrorMessage),
        (status = 403, body = ErrorMessage),
    )
))]
async fn start_machine(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ServerName>,
) -> Result<(StatusCode, Json<Server>), Error> {
    match state.drivers.get(&req.name) {
        Some(driver) => {
            driver.start()?;
            let status = driver.status()?;
            Ok((
                StatusCode::ACCEPTED,
                Json(Server {
                    name: status.name,
                    hostname: status.hostname,
                    running: status.running,
                    reason: status.reason,
                }),
            ))
        }
        None => Err(Error::NotFound("driver is not found".into())),
    }
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    put,
    path = "/api/servers/stop",
    request_body = ServerName,
    responses(
        (status = 202, body = Server),
        (status = 400, body = ErrorMessage),
        (status = 401, body = ErrorMessage),
        (status = 403, body = ErrorMessage),
    )
))]
async fn stop_machine(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ServerName>,
) -> Result<(StatusCode, Json<Server>), Error> {
    match state.drivers.get(&req.name) {
        Some(driver) => {
            driver.stop()?;
            let status = driver.status()?;
            Ok((
                StatusCode::ACCEPTED,
                Json(Server {
                    name: status.name,
                    hostname: status.hostname,
                    running: status.running,
                    reason: status.reason,
                }),
            ))
        }
        None => Err(Error::NotFound("driver is not found".into())),
    }
}
