use rusty_pake::server::run;
use std::env;

#[tokio::main]
async fn main() {
    let id = env::var("SERVER_ID").unwrap_or("id".into());
    println!("starting server with id: {}", &id);

    run("0.0.0.0:3000", &id).await;
}
