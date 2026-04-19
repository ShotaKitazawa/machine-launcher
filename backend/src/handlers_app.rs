use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

use machine_launcher_common::{Endpoint, ListServers, Server, ServerName, StartServer, StopServer};

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
