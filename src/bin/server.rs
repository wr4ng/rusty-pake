use rusty_pake::server::run;
use std::env;

#[tokio::main]
async fn main() {
    let id = env::var("SERVER_ID").unwrap_or("SPAKE2+".into());
    println!("starting server with id: {}", &id);

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    run("0.0.0.0:3000", &id).await;
}
