use rusty_pake::client;
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let server_ip = prompt_default("Enter server IP", "http://localhost:3000");

    let server_id = match client::get_server_id(&server_ip).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to get server id: {:?}", e);
            return;
        }
    };
    println!("Retrieved server id: {}", &server_id);

    let mut saved_id: Option<String> = None;
    let mut saved_key: Option<String> = None;

    println!();
    loop {
        let action = prompt("Action (setup, exchange, verify, exit):").unwrap_or("".into());
        match action.as_str() {
            "setup" => {
                let client_id = prompt_saved("Enter client ID:", saved_id.as_deref())
                    .expect("need to provide client id!");
                saved_id = Some(client_id.clone());
                let password = prompt("Enter password:").expect("need to enter password!");
                if let Err(e) =
                    client::perform_setup(&server_ip, &server_id, &client_id, &password).await
                {
                    eprintln!("Error during setup: {}", e);
                }
            }
            "exchange" => {
                let client_id = prompt_saved("Enter client ID:", saved_id.as_deref())
                    .expect("need to provide client id!");
                saved_id = Some(client_id.clone());
                let password = prompt("Enter password:").expect("need to enter password!");

                match client::perform_exchange(&server_ip, &server_id, &client_id, &password).await
                {
                    Ok(key) => {
                        saved_key = Some(key);
                    }
                    Err(e) => {
                        eprintln!("Error during exchange: {}", e);
                    }
                }
            }
            "verify" => {
                let client_id = prompt_saved("Enter client ID:", saved_id.as_deref())
                    .expect("need to provide client id!");
                saved_id = Some(client_id.clone());
                let key =
                    prompt_saved("Enter key", saved_key.as_deref()).expect("need to enter key!");
                if let Err(e) = client::perform_verify(&server_ip, &client_id, &key).await {
                    eprintln!("Error during exchange: {}", e);
                }
            }
            "exit" => {
                return;
            }
            _ => {
                eprintln!("invalid protocol state");
                return;
            }
        }
    }
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

fn prompt_default(msg: &str, default: &str) -> String {
    prompt(&format!("{} ({}):", msg, default)).unwrap_or(default.to_string())
}

fn prompt_saved(msg: &str, saved: Option<&str>) -> Option<String> {
    match saved {
        Some(saved) => Some(prompt_default(msg, saved)),
        None => prompt(msg),
    }
}
