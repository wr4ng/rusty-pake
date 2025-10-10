use rusty_pake::{
    pake::{client_cipher, client_secret},
    shared::SetupRequest,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let server_ip = prompt("Enter server IP");
    let client_id = prompt("Enter client ID");
    let password = prompt("Enter password");

    let protocol_state = prompt("\nChoose protocol state (setup or login)");
    match protocol_state.as_str() {
        "setup" => {
            if let Err(e) = handle_setup(&server_ip, &client_id, &password).await {
                eprintln!("Error during setup: {}", e);
            }
        }
        "login" => todo!("implement login"),
        _ => {
            eprintln!("invalid protocol state. Choose 'setup' or 'login'");
            return;
        }
    }
}

async fn handle_setup(
    server_ip: &str,
    client_id: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    println!("Starting PAKE setup process...");

    //TODO: Get server id from server_ip/id
    let server_id = "id";

    // Perform client setup
    let (phi0, phi1) = client_secret(password, client_id, server_id);
    let c = client_cipher(phi1);

    // Create request
    let request = SetupRequest::new(
        client_id.to_string(),
        &phi0.to_bytes(),
        &c.compress().to_bytes(),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&request)?;
    println!("Sending setup request to server...");

    // Send to server
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/setup", server_ip))
        .header("Content-Type", "application/json")
        .body(json)
        .send()
        .await?;

    // Handle response
    if res.status().is_success() {
        println!("Setup completed successfully!");
        Ok(())
    } else {
        anyhow::bail!("Server returned error: {}", res.status());
    }
}

fn prompt(msg: &str) -> String {
    print!("{}: ", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
