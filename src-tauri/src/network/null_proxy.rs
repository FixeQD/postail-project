/// Null proxy server — accepts connections and returns 502 for every request.
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{info, warn};

const RESPONSE_502: &[u8] =
    b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

pub async fn start_on_port(port: u16) {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            warn!("null proxy: failed to bind on port {port}: {e}");
            return;
        }
    };

    info!("null proxy: listening on 127.0.0.1:{port}");

    loop {
        match listener.accept().await {
            Ok((mut stream, _peer)) => {
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_millis(500),
                        tokio::io::AsyncReadExt::read(&mut stream, &mut buf),
                    )
                    .await;
                    let _ = stream.write_all(RESPONSE_502).await;
                });
            }
            Err(e) => {
                warn!("null proxy: accept error: {e}");
            }
        }
    }
}
