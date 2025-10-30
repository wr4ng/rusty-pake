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
use curve25519_dalek::{RistrettoPoint, Scalar, ristretto::CompressedRistretto};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::{
    pake::{server_compute_key, server_initial},
    shared::{LoginRequest, LoginResponse, SetupRequestEncoded, VerifyRequestEncoded},
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
            error!("failed to lock sessions");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!(id = %request.id, phi0 = %request.phi0, c = %request.c, "/setup");

    let request = match request.decode() {
        Ok(r) => r,
        Err(error) => {
            error!(%error, "failed to decode request");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if sessions.contains_key(&request.id) {
        error!(id = %request.id, "client id is already setup");
        return Err(StatusCode::BAD_GATEWAY);
    }

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
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut sessions = match state.sessions.lock() {
        Ok(s) => s,
        _ => {
            error!("failed to lock sessions");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!(id = %request.id, u = %request.u, "/login");

    let session = sessions.get_mut(&request.id).ok_or_else(|| {
        info!(id = %request.id, "client session not found");
        StatusCode::UNAUTHORIZED
    })?;

    // decode u
    let u_bytes = hex::decode(&request.u).map_err(|_| StatusCode::BAD_REQUEST)?;
    let u_comp = CompressedRistretto::from_slice(&u_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    let u_point = u_comp.decompress().ok_or(StatusCode::BAD_REQUEST)?;

    // server step
    let (v_point, beta) = server_initial(session.phi0);

    // derive key
    let idc = &request.id;
    let ids = &state.id;
    let k_s = server_compute_key(idc, ids, session.phi0, session.c, beta, u_point, v_point);
    session.key = Some(k_s);

    let v_hex = hex::encode(v_point.compress().to_bytes());
    Ok(Json(LoginResponse {
        v: v_hex,
        id_s: state.id.clone(),
    }))
}

async fn handle_verify(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequestEncoded>,
) -> Result<(), StatusCode> {
    let sessions = match state.sessions.lock() {
        Ok(s) => s,
        _ => {
            error!("failed to lock sessions");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let client_session = sessions.get(&request.idc).ok_or_else(|| {
        info!(id = %request.idc, "client session not found");
        StatusCode::BAD_REQUEST
    })?;
    let stored_key = client_session.key.ok_or_else(|| {
        info!(id = %request.idc, "not key stored for client");
        StatusCode::BAD_REQUEST
    })?;
    let request = request.decode().map_err(|error| {
        info!(%error, "failed to decode request");
        StatusCode::BAD_REQUEST
    })?;

    if stored_key != request.key {
        info!(
            id = %request.idc,
            provided_key = %hex::encode(request.key),
            stored_key = %hex::encode(stored_key),
            "verification failed!"
        );

        return Err(StatusCode::UNAUTHORIZED);
    }
    info!(id = %request.idc, "verification succeeded");
    Ok(())
}
