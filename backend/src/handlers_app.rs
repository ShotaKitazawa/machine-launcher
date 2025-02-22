use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

use crate::{AppState, Error};

pub fn routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/servers", get(machine_status))
        .route("/servers/start", put(start_machine))
        .route("/servers/stop", put(stop_machine))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state,
            crate::middlewares::auth_middleware,
        ))
}

#[derive(Debug, serde::Serialize)]
struct MachineStatusResponseOne {
    name: String,
    hostname: String,
    running: bool,
    reason: Option<String>,
}

async fn machine_status(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<MachineStatusResponseOne>>), Error> {
    let mut res: Vec<MachineStatusResponseOne> = vec![];
    for driver in state.drivers.clone().values() {
        let status = driver.status()?;
        res.push(MachineStatusResponseOne {
            name: status.name,
            hostname: status.hostname,
            running: status.running,
            reason: status.reason,
        });
    }
    Ok((StatusCode::OK, Json(res)))
}

#[derive(Debug, serde::Deserialize)]
struct StartMachineRequest {
    name: String,
}

async fn start_machine(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartMachineRequest>,
) -> Result<(StatusCode, Json<MachineStatusResponseOne>), Error> {
    match state.drivers.get(&req.name) {
        Some(driver) => {
            driver.start()?;
            let status = driver.status()?;
            Ok((
                StatusCode::ACCEPTED,
                Json(MachineStatusResponseOne {
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

#[derive(Debug, serde::Deserialize)]
struct StopMachineRequest {
    name: String,
}

async fn stop_machine(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StopMachineRequest>,
) -> Result<(StatusCode, Json<MachineStatusResponseOne>), Error> {
    match state.drivers.get(&req.name) {
        Some(driver) => {
            driver.stop()?;
            let status = driver.status()?;
            Ok((
                StatusCode::ACCEPTED,
                Json(MachineStatusResponseOne {
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
