// actual udp network implementation

use std::net::SocketAddr;

use tokio::net::UdpSocket;


async fn reply(socket: &UdpSocket, dst: SocketAddr, msg: Vec<u8>) {
    if let Err(e) = socket.send_to(&msg, dst).await {
        tracing::debug!("send_to {dst} failed: {e}");
    }
}
