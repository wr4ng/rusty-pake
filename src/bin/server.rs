use axum::Json;
use axum::extract::State;
use axum::routing::get;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::{RistrettoPoint, Scalar};
use rusty_pake::pake::{server_compute_key, server_initial};
use rusty_pake::shared::{LoginRequest, LoginResponse, SetupRequest, VerifyRequest};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

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

#[tokio::main]
async fn main() {
    let id = env::var("SERVER_ID").unwrap_or("id".into());
    println!("starting server with id: {}", &id);

    let appstate = AppState {
        id: id,
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/id", get(handle_id))
        .route("/setup", post(handle_setup))
        .route("/login", post(handle_login))
        .route("/verify", post(handle_verify))
        .with_state(appstate);

    println!("listening on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_id(State(state): State<AppState>) -> impl IntoResponse {
    state.id
}

async fn handle_setup(
    State(state): State<AppState>,
    Json(request): Json<SetupRequest>,
) -> impl IntoResponse {
    let mut sessions = match state.sessions.lock() {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let (id, phi0, c) = request.decode().unwrap(); // TODO: Handle errors
    println!("Setup request received from client ID: {}", &id);

    println!("phi0={:?}\nc={:?}", phi0, c);

    let phi0 = Scalar::from_bytes_mod_order(phi0);
    let c = CompressedRistretto::from_slice(&c)
        .unwrap()
        .decompress()
        .unwrap(); // TODO: Handle errors

    if sessions.contains_key(&id) {
        //TODO: Handle client trying to setup again
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sessions.insert(
        id,
        Session {
            phi0: phi0,
            c: c,
            key: None,
        },
    );

    StatusCode::OK
}

async fn handle_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sess = sessions.get_mut(&req.id).ok_or(StatusCode::UNAUTHORIZED)?;

    // decode u
    let u_bytes = hex::decode(&req.u).map_err(|_| StatusCode::BAD_REQUEST)?;
    let u_comp = CompressedRistretto::from_slice(&u_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    let u_point = u_comp.decompress().ok_or(StatusCode::BAD_REQUEST)?;

    // server step
    let (v_point, beta) = server_initial(sess.phi0);

    // derive key
    let idc = &req.id;
    let ids = &state.id;
    let k_s = server_compute_key(idc, ids, sess.phi0, sess.c, beta, u_point, v_point);
    sess.key = Some(k_s);

    let v_hex = hex::encode(v_point.compress().to_bytes());
    Ok(Json(LoginResponse {
        v: v_hex,
        id_s: state.id.clone(),
    }))
}

async fn handle_verify(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequest>,
) -> Result<(), StatusCode> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (idc, key) = request.decode().map_err(|_| StatusCode::BAD_REQUEST)?;
    let client_session = sessions.get(&idc).ok_or(StatusCode::BAD_REQUEST)?;
    let stored_key = client_session.key.ok_or(StatusCode::BAD_REQUEST)?;

    if !(stored_key == key) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}
