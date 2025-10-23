use curve25519_dalek::ristretto::CompressedRistretto;

use rusty_pake::{
    pake::{client_cipher, client_compute_key, client_initial, client_secret},
    shared::{LoginRequest, LoginResponse, SetupRequest, VerifyRequestEncoded},
};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let server_ip = prompt("Enter server IP (http://localhost:3000):")
        .unwrap_or("http://localhost:3000".into());

    let server_id = match get_server_id(&server_ip).await {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Failed to get server id: {:?}", e);
            return;
        }
    };

    loop {
        let protocol_state =
            prompt("\nAction (setup, login, verify, exit):").expect("empty protocol");
        match protocol_state.as_str() {
            "setup" => {
                let client_id = prompt("Enter client ID:").expect("need to provide client id!");
                let password = prompt("Enter password:").expect("need to enter password!");
                if let Err(e) = handle_setup(&server_ip, &server_id, &client_id, &password).await {
                    eprintln!("Error during setup: {}", e);
                }
            }
            "login" => {
                //TODO: Save key to reuse in verify
                let client_id = prompt("Enter client ID:").expect("need to provide client id!");
                let password = prompt("Enter password:").expect("need to enter password!");
                if let Err(e) = handle_login(&server_ip, &server_id, &client_id, &password).await {
                    eprintln!("Error during login: {}", e);
                }
            }
            "verify" => {
                let client_id = prompt("Enter client ID:").expect("need to provide client id!");
                let key = prompt("Enter key:").expect("need to enter key!");
                if let Err(e) = handle_verify(&server_ip, &client_id, key).await {
                    eprintln!("Error during login: {}", e);
                }
            }
            "exit" => {
                return;
            }
            _ => {
                eprintln!("invalid protocol state. Choose 'setup' or 'login'");
                return;
            }
        }
    }
}

async fn handle_setup(
    server_ip: &str,
    server_id: &str,
    client_id: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    println!("Starting PAKE setup process...");

    // Perform client setup
    let (phi0, phi1) = client_secret(password, client_id, server_id);
    let c = client_cipher(phi1);

    // Create request
    let request = SetupRequest::new(client_id.to_string(), phi0, c);

    // Serialize to JSON
    let json = serde_json::to_string(&request.encode())?;
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

async fn handle_login(
    server_ip: &str,
    server_id: &str,
    idc: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    // client secrets & initial message
    let (phi0, phi1) = client_secret(password, idc, server_id);
    let (u_point, alpha) = client_initial(phi0);

    // POST /login with hex(u)
    let req = LoginRequest {
        id: idc.to_string(),
        u: hex::encode(u_point.compress().to_bytes()),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/login", server_ip))
        .json(&req)
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("server returned {}", resp.status());
    }

    // parse response and compute k on client
    let lr: LoginResponse = resp.json().await?;
    let v_bytes = hex::decode(lr.v)?;
    let v_point = CompressedRistretto::from_slice(&v_bytes)
        .map_err(|_| anyhow::anyhow!("bad v length"))?
        .decompress()
        .ok_or_else(|| anyhow::anyhow!("bad v point"))?;

    let k_c = client_compute_key(idc, server_id, phi0, phi1, alpha, u_point, v_point);
    println!("Login completed\nkey={}", hex::encode(k_c));
    Ok(())
}

async fn handle_verify(server_ip: &str, idc: &str, key: String) -> Result<(), anyhow::Error> {
    let request = VerifyRequestEncoded::new(idc.to_string(), key);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/verify", server_ip))
        .json(&request)
        .send()
        .await?;

    match response.status().is_success() {
        true => println!("Verification successful"),
        false => println!("Verification failed!"),
    }

    Ok(())
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
