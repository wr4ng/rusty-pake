use rusty_pake::client;

#[tokio::main]
async fn main() {
    let ip = "http://localhost:3000";

    let clients = vec![
        ("Alice", "ilovebob123"),
        ("Bob", "alice1234"),
        ("Charlie", "thirdwheel!"),
        ("Eve", "ijustwantfriends123"),
    ];

    let handles: Vec<_> = clients
        .into_iter()
        .map(|(id, password)| {
            tokio::spawn(async move {
                let server_id = client::get_server_id(ip).await?;
                client::perform_setup(ip, &server_id, id, password).await?;
                for _ in 0..20 {
                    let key = client::perform_exchange(ip, &server_id, &id, &password).await?;
                    client::perform_verify(ip, &id, &key).await.unwrap();
                }
                Ok::<bool, anyhow::Error>(true)
            })
        })
        .collect();

    for handle in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(_) => (),
                Err(error) => println!("{}", error),
            },
            Err(join_error) => println!("{}", join_error),
        }
    }
}
