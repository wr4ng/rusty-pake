use reqwest;
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let server = prompt("Enter server IP:");
    let client_id = prompt("Enter client ID:");
    let password = prompt("Enter password:");

    print!("Connecting to server at {}...\n", server);
    println!("Client ID: {}, Password: {}", client_id, password);

    // connect to server given ip
    let addr = format!("http://127.0.0.1:{}/message", server); // Fix
    let client = reqwest::Client::new();
    // send initial message to server
    let res = client.post(&addr).body("Hello from client").send().await;
    match res {
        Ok(response) => {
            let text = response.text().await.unwrap();
            println!("Received from server: {}", text);
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return;
        }
    }

    // perform PAKE protocol
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
