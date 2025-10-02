use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    print!("Server listening on {}", listener.local_addr()?);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> tokio::io::Result<()> {
    todo!("Implement server logic here");
}
