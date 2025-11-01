use rusty_pake::client;

#[tokio::main]
async fn main() {
    let ip = "http://localhost:3000";
    let server_id = "id";
    let client_id = "Alice";
    let password = "ilovebob123";

    client::perform_setup(ip, server_id, client_id, password)
        .await
        .unwrap();

    let key = client::perform_login(ip, server_id, client_id, password)
        .await
        .unwrap();

    client::perform_verify(ip, client_id, &key).await.unwrap();
}
