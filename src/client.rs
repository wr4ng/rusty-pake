use curve25519_dalek::ristretto;

use crate::{pake, shared};

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
    let (phi0, phi1) = pake::client_secret(password, client_id, server_id);
    let c = pake::client_cipher(phi1);

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
    let (phi0, phi1) = pake::client_secret(password, idc, server_id);
    let (u_point, alpha) = pake::client_initial(phi0);

    // POST /login with hex(u)
    let req = shared::LoginRequest {
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
    let lr: shared::LoginResponse = resp.json().await?;
    let v_bytes = hex::decode(lr.v)?;
    let v_point = ristretto::CompressedRistretto::from_slice(&v_bytes)
        .map_err(|_| anyhow::anyhow!("bad v length"))?
        .decompress()
        .ok_or_else(|| anyhow::anyhow!("bad v point"))?;

    let k_c = pake::client_compute_key(idc, server_id, phi0, phi1, alpha, u_point, v_point);
    let key = hex::encode(k_c);
    println!("Login completed\nkey={}", key);
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
