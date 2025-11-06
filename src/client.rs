use crate::{shared, spake2plus};

pub async fn get_server_id(server_ip: &str) -> Result<String, anyhow::Error> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/id", server_ip)).send().await?;
    let id = response.text().await?;
    Ok(id)
}

pub async fn perform_setup(
    server_ip: &str,
    server_id: &str,
    client_id: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    println!("Starting PAKE setup process...");

    // Perform client setup
    let (phi0, phi1) = spake2plus::client_secret(password, client_id, server_id);
    let c = spake2plus::client_cipher(phi1);

    // Create request
    let request = shared::SetupRequest::new(client_id.to_string(), phi0, c);

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

pub async fn perform_login(
    server_ip: &str,
    server_id: &str,
    idc: &str,
    password: &str,
) -> Result<String, anyhow::Error> {
    // client secrets & initial message
    let (phi0, phi1) = spake2plus::client_secret(password, idc, server_id);
    let (u, alpha) = spake2plus::client_initial(phi0);

    // POST /login with hex(u)
    let request = shared::LoginRequest::new(idc.to_string(), u);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/login", server_ip))
        .json(&request.encode())
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("server returned {}", response.status());
    }

    // parse response and compute k on client
    let response: shared::LoginResponseEncoded = response.json().await?;
    let response = response.decode()?;

    let k_c = spake2plus::client_compute_key(idc, server_id, phi0, phi1, alpha, u, response.v);
    let key = hex::encode(k_c);
    println!(
        "Login completed\nalpha={}\nu={}\nkey={}",
        hex::encode(alpha.as_bytes()),
        hex::encode(u.compress().as_bytes()),
        key
    );
    Ok(key)
}

pub async fn perform_verify(server_ip: &str, idc: &str, key: &str) -> Result<bool, anyhow::Error> {
    let request = shared::VerifyRequestEncoded::new(idc.to_string(), key.to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/verify", server_ip))
        .json(&request)
        .send()
        .await?;

    let success = response.status().is_success();

    match success {
        true => println!("Verification successful"),
        false => println!("Verification failed!"),
    }

    Ok(success)
}
