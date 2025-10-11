use rusty_pake::{
    pake::{client_cipher, client_secret},
    shared::SetupRequest,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let server_ip = prompt("Enter server IP (http://localhost:3000):")
        .unwrap_or("http://localhost:3000".into());
    let client_id = prompt("Enter client ID:").expect("need to provide client id!");
    let password = prompt("Enter password:").expect("need to enter password!");

    let protocol_state =
        prompt("\nChoose protocol state (setup or login):").expect("empty protocol");
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

    let server_id = get_server_id(server_ip).await?;

    // Perform client setup
    let (phi0, phi1) = client_secret(password, client_id, &server_id);
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

async fn get_server_id(server_ip: &str) -> Result<String, anyhow::Error> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/id", server_ip)).send().await?;
    let id = response.text().await?;
    Ok(id)
}

fn prompt(msg: &str) -> Option<String> {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    match input.trim() {
        "" => None,
        s => Some(s.into()),
    }
}
