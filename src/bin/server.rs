use rusty_pake::server::run;
use std::env;

#[tokio::main]
async fn main() {
    let id = env::var("SERVER_ID").unwrap_or("SPAKE2+".into());
    let port = env::var("PORT")
        .map(|p| p.parse::<u32>().unwrap())
        .unwrap_or(3000);

    println!("starting server with id: {}", &id);

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    run(port, &id).await;
}
