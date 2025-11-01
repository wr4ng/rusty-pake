use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use curve25519_dalek::{RistrettoPoint, Scalar};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::{
    pake::{server_compute_key, server_initial},
    shared::{
        LoginRequestEncoded, LoginResponse, LoginResponseEncoded, SetupRequestEncoded,
        VerifyRequestEncoded,
    },
};

#[derive(Clone)]
struct AppState {
    id: String,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

struct Session {
    phi0: Scalar,
    c: RistrettoPoint,
    key: Option<[u8; 32]>,
}

pub async fn run(address: &str, id: &str) {
    let appstate = AppState {
        id: id.to_string(),
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/id", get(handle_id))
        .route("/setup", post(handle_setup))
        .route("/login", post(handle_login))
        .route("/verify", post(handle_verify))
        .with_state(appstate)
        .layer(TraceLayer::new_for_http());

    println!("listening on http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_id(State(state): State<AppState>) -> impl IntoResponse {
    info!("/id");
    state.id
}

async fn handle_setup(
    State(state): State<AppState>,
    Json(request): Json<SetupRequestEncoded>,
) -> Result<(), StatusCode> {
    let mut sessions = match state.sessions.lock() {
        Ok(s) => s,
        _ => {
            error!("/setup failed to lock sessions");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let request = match request.decode() {
        Ok(r) => r,
        Err(error) => {
            error!(%error, "/setup failed to decode request");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if sessions.contains_key(&request.id) {
        error!(id = %request.id, "/setup client id is already setup");
        return Err(StatusCode::BAD_GATEWAY);
    }

    info!(
        id = %request.id,
        phi0 = %hex::encode(request.phi0.as_bytes()),
        c = %hex::encode(request.c.compress().as_bytes()),
        "/setup completed"
    );

    sessions.insert(
        request.id,
        Session {
            phi0: request.phi0,
            c: request.c,
            key: None,
        },
    );
    Ok(())
}

async fn handle_login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequestEncoded>,
) -> Result<Json<LoginResponseEncoded>, StatusCode> {
    let mut sessions = match state.sessions.lock() {
        Ok(s) => s,
        _ => {
            error!("/login failed to lock sessions");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let session = sessions.get_mut(&request.id).ok_or_else(|| {
        info!(id = %request.id, "/login client session not found");
        StatusCode::UNAUTHORIZED
    })?;

    let request = match request.decode() {
        Ok(r) => r,
        Err(error) => {
            error!(%error, "/login failed to decode request");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // server step
    let (v, beta) = server_initial(session.phi0);
    info!(
        id = %request.id,
        u = %hex::encode(request.u.compress().as_bytes()),
        v = %hex::encode(v.compress().as_bytes()),
        beta = %hex::encode(beta.as_bytes()),
        "/login completed"
    );

    // derive key
    let k_s = server_compute_key(
        &request.id,
        &state.id,
        session.phi0,
        session.c,
        beta,
        request.u,
        v,
    );
    // store key in client session
    session.key = Some(k_s);

    Ok(Json(LoginResponse::new(v).encode()))
}

async fn handle_verify(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequestEncoded>,
) -> Result<(), StatusCode> {
    let sessions = match state.sessions.lock() {
        Ok(s) => s,
        _ => {
            error!("/verify failed to lock sessions");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let client_session = sessions.get(&request.idc).ok_or_else(|| {
        info!(id = %request.idc, "/verify client session not found");
        StatusCode::BAD_REQUEST
    })?;
    let stored_key = client_session.key.ok_or_else(|| {
        info!(id = %request.idc, "/verify not key stored for client");
        StatusCode::BAD_REQUEST
    })?;
    let request = request.decode().map_err(|error| {
        info!(%error, "/verify failed to decode request");
        StatusCode::BAD_REQUEST
    })?;

    if stored_key != request.key {
        info!(
            id = %request.idc,
            provided_key = %hex::encode(request.key),
            stored_key = %hex::encode(stored_key),
            "/verify verification failed!"
        );

        return Err(StatusCode::UNAUTHORIZED);
    }
    info!(id = %request.idc, "/verify verification succeeded");
    Ok(())
}
