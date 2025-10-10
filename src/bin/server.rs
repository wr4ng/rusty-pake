use axum::Json;
use axum::routing::get;
use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
use curve25519_dalek::Scalar;
use curve25519_dalek::ristretto::CompressedRistretto;
use rusty_pake::shared::SetupRequest;
use std::env;

#[tokio::main]
async fn main() {
    let id = env::var("SERVER_ID").unwrap_or("id".into());
    println!("starting server with id: {}", &id);

    let app = Router::new()
        .route("/id", get(handle_id))
        .route("/setup", post(handle_setup))
        .route("/login", post(handle_login))
        .route("/verify", post(handle_verify));

    println!("listening on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_id() {
    todo!("implement verify")
}

async fn handle_setup(Json(request): Json<SetupRequest>) -> impl IntoResponse {
    let (id, phi0, c) = request.decode().unwrap(); // TODO: Handle errors
    println!("Setup request received from client ID: {}", &id);

    println!("phi0={:?}\n, c={:?}", phi0, c);

    let _phi0 = Scalar::from_bytes_mod_order(phi0);
    let _c = CompressedRistretto::from_slice(&c)
        .unwrap()
        .decompress()
        .unwrap(); // TODO: Handle errors

    // TODO Store client id, phi0 and c

    StatusCode::OK
}

async fn handle_login() {
    todo!("implement login")
}

async fn handle_verify() {
    todo!("implement verify")
}
