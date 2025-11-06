use std::sync::Once;

use rusty_pake::{client, server};
use serial_test::serial;

static INIT: Once = Once::new();

async fn setup_server(id: &str) {
    let id = id.to_string();

    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_test_writer() // ensures logs go to test output
            .compact()
            .init();
    });

    tokio::spawn(async move {
        server::run("0.0.0.0:3000", &id).await;
    });

    let client = reqwest::Client::new();
    for _ in 0..20 {
        if client.get("http://localhost:3000/id").send().await.is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("Server failed to start in time");
}

#[tokio::test]
#[serial]
async fn test_get_server_id() {
    let server_id = "test-id";
    setup_server(server_id).await;

    let retrieved_id = client::get_server_id("http://localhost:3000").await;
    assert_eq!(retrieved_id.unwrap(), server_id)
}

#[tokio::test]
#[serial]
async fn test_successful_exchange() {
    let ip = "http://localhost:3000";
    let server_id = "id";
    let client_id = "Alice";
    let password = "ilovebob123";

    setup_server(server_id).await;

    client::perform_setup(ip, server_id, client_id, password)
        .await
        .unwrap();

    let key = client::perform_exchange(ip, server_id, client_id, password)
        .await
        .unwrap();

    client::perform_verify(ip, client_id, &key).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_wrong_password_exchange() {
    let ip = "http://localhost:3000";
    let server_id = "id";
    let client_id = "Bob";
    let password = "alice1234";

    setup_server(server_id).await;

    client::perform_setup(ip, server_id, client_id, password)
        .await
        .unwrap();

    let wrong_password = "alice1234oops";
    let key = client::perform_exchange(ip, server_id, client_id, wrong_password)
        .await
        .unwrap();

    let success = client::perform_verify(ip, client_id, &key).await.unwrap();
    assert_eq!(success, false)
}

#[tokio::test]
#[serial]
async fn test_multiple_exchanges() {
    let ip = "http://localhost:3000";
    let server_id = "popular-server";
    let client_id = "Bob";
    let password = "alice1234";

    setup_server(server_id).await;

    client::perform_setup(ip, server_id, client_id, password)
        .await
        .unwrap();

    // Exchange 1
    let key1 = client::perform_exchange(ip, server_id, client_id, password)
        .await
        .unwrap();

    let success1 = client::perform_verify(ip, client_id, &key1).await.unwrap();

    // Exchange 2
    let key2 = client::perform_exchange(ip, server_id, client_id, password)
        .await
        .unwrap();

    let success2 = client::perform_verify(ip, client_id, &key2).await.unwrap();

    assert!(success1);
    assert!(success2);
    assert_ne!(key1, key2);
}

#[tokio::test]
#[serial]
async fn test_multiple_clients() {
    let ip = "http://localhost:3000";
    let server_id = "id";
    let client_a = "Alice";
    let password_a = "ilovebob123";
    let client_b = "Bob";
    let password_b = "alice1234";

    setup_server(server_id).await;

    // Alice setup
    client::perform_setup(ip, server_id, client_a, password_a)
        .await
        .unwrap();

    // Bob setup
    client::perform_setup(ip, server_id, client_b, password_b)
        .await
        .unwrap();

    // Alice exchange
    let key_a = client::perform_exchange(ip, server_id, client_a, password_a)
        .await
        .unwrap();

    // Bob exchange
    let key_b = client::perform_exchange(ip, server_id, client_b, password_b)
        .await
        .unwrap();

    // Alice verify
    let success_a = client::perform_verify(ip, client_a, &key_a).await.unwrap();
    assert_eq!(success_a, true);

    // Bob verify
    let success_b = client::perform_verify(ip, client_b, &key_b).await.unwrap();
    assert_eq!(success_b, true);
}
