use axum::body::to_bytes;
use axum::routing::get;
use axum::{Router, body::Body, http::StatusCode, response::IntoResponse, routing::post};
use curve25519_dalek::Scalar;
use rusty_pake::pake::client_cipher;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

#[derive(Serialize, Deserialize)]
struct SetupRequest {
    client_id: String,
    phi0_bytes: [u8; 32],
    phi1_bytes: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct SetupResponse {
    phi0_bytes: [u8; 32],
    c_bytes: [u8; 32],
}

#[tokio::main]
async fn main() {
    let id = match env::var("SERVER_ID") {
        Ok(id) => id,
        Err(_) => {
            eprintln!("No SERVER_ID provided!");
            return;
        }
    };
    println!("starting server with id: {}", &id);

    let app = Router::new()
        .route("/id", get(handle_id))
        .route("/setup", post(handle_setup))
        .route("/login", post(handle_login))
        .route("/verify", post(handle_verify));

    println!("Listening on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_id() {
    todo!("implement verify")
}

async fn handle_setup(body: Body) -> impl IntoResponse {
    let bytes = to_bytes(body, 1024 * 1024).await.unwrap();

    // Try to parse as JSON
    match serde_json::from_slice::<SetupRequest>(&bytes) {
        Ok(request) => {
            println!(
                "Setup request received from client ID: {}",
                request.client_id
            );

            // Convert bytes back to Scalar objects
            let phi0 = Scalar::from_bytes_mod_order(request.phi0_bytes);
            let phi1 = Scalar::from_bytes_mod_order(request.phi1_bytes);

            // Execute setup_2 to generate c
            let c = client_cipher(phi1);

            // Create response
            let response = SetupResponse {
                phi0_bytes: phi0.to_bytes(),
                c_bytes: c.compress().to_bytes(),
            };

            // Store client credentials for later (in a real implementation)
            // This would be saved in a database
            println!("Generated c for client {}: {:?}", request.client_id, c);

            // Return the serialized response
            match serde_json::to_string(&response) {
                Ok(json) => (StatusCode::OK, json),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error serializing response".to_string(),
                ),
            }
        }
        Err(e) => {
            println!("Error parsing setup request: {}", e);
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid request format: {}", e),
            )
        }
    }
}

async fn handle_login(body: Body) -> impl IntoResponse {
    let bytes = to_bytes(body, 1024 * 1024).await.unwrap(); // 1 MB limit
    println!("Login received: {:?}", bytes);

    // Placeholder response
    b"Login ACK".to_vec()
}

async fn handle_verify() {
    todo!("implement verify")
}
