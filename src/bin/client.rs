use curve25519_dalek::{RistrettoPoint, Scalar}; // Is this needed here?
use reqwest;
use rusty_pake::pake::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize, Deserialize)]
struct SetupRequest {
    client_id: String,
    phi0_bytes: [u8; 32],
    phi1_bytes: [u8; 32],
    server_id: String,
}
#[derive(Serialize, Deserialize)]
struct SetupResponse {
    phi0_bytes: [u8; 32],
    c_bytes: [u8; 32],
}

#[tokio::main]
async fn main() {
    let server_ip = prompt("Enter server IP:");
    let client_id = prompt("Enter client ID:");
    let password = prompt("Enter password:");

    print!("Connecting to server at {}...\n", server_ip);
    println!("Client ID: {}, Password: {}", client_id, password);

    println!("Choose protocol state (message, setup or login)");
    let protocol_state = prompt("Enter protocol state:");

    match protocol_state.as_str() {
        "message" => {
            // Original test of message handling
            if let Err(e) = message_test(&server_ip, &protocol_state).await {
                eprintln!("Error during message test: {}", e);
            }
        }
        "setup" => {
            if let Err(e) = handle_setup(&server_ip, &client_id, &password).await {
                eprintln!("Error during setup: {}", e);
            }
        }
        "login" => (),
        _ => {
            eprintln!("Invalid protocol state. Choose 'message', 'setup' or 'login'.");
            return;
        }
    }
}

async fn handle_setup(
    server_ip: &str,
    client_id: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_id = "server"; // Usually this would be known or discovered

    println!("Starting PAKE setup process...");

    // Generate phi0 and phi1 using setup_1
    let (phi0, phi1) = setup_1(password, client_id, server_id);

    // Create request data with serialized scalars
    let request = SetupRequest {
        phi0_bytes: phi0.to_bytes(),
        phi1_bytes: phi1.to_bytes(),
        client_id: client_id.to_string(),
        server_id: server_id.to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&request)?;
    println!("Sending setup request to server...");

    // Send to server
    let client = reqwest::Client::new();
    let addr = format!("http://127.0.0.1:{}/setup", server_ip);
    let res = client
        .post(&addr)
        .header("Content-Type", "application/json")
        .body(json)
        .send()
        .await?;

    // Handle response
    if res.status().is_success() {
        // let response_json = res.text().await?;
        // let setup_response: SetupResponse = serde_json::from_str(&response_json)?;

        // // Deserialize received values
        // let received_phi0 = Scalar::from_bytes_mod_order(setup_response.phi0_bytes);
        // let received_c = RistrettoPoint::from_bytes(&setup_response.c_bytes).unwrap();

        // println!("Setup completed successfully!");
        // println!("Received values from server:");
        // println!("phi0: {:?}", received_phi0);
        // println!("c: {:?}", received_c);

        // Store these values for later use in login phase
        // In a real implementation, you'd persist these securely

        println!("Setup completed successfully!");
        Ok(())
    } else {
        Err(format!("Server returned error: {}", res.status()).into())
    }
}

async fn message_test(
    server: &str,
    protocol_state: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{}/{}", server, protocol_state); // Fix
    let client = reqwest::Client::new();
    // send initial message to server
    let res = client.post(&addr).body("Hello from client").send().await;
    match res {
        Ok(response) => {
            let text = response.text().await?;
            println!("Received from server: {}", text);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            Err(Box::new(e))
        }
    }
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
