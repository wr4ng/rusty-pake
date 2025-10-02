use axum::body::to_bytes;
use axum::{Router, body::Body, response::IntoResponse, routing::post};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/message", post(handle_message));

    println!("Listening on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn handle_message(body: Body) -> impl IntoResponse {
    let bytes = to_bytes(body, 1024 * 1024).await.unwrap(); // 1 MB limit
    println!("Server received: {:?}", bytes);

    // Example: just echo back with "ACK"
    b"ACK".to_vec()
}
