use std::net::{SocketAddr};
use tokio::net::TcpStream;

pub async fn wait_for_port(addr: SocketAddr) {
    loop {
        if TcpStream::connect(addr).await.is_ok() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Sleep for a second
    }
}

