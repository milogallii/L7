use tokio::net::UdpSocket;
use anyhow::{Result, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let destination = "10.42.0.60:8888";
    let packet_size: usize = 65507;

    let prefix = "$IIHDT,33,T*11";

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let mut packet = vec![b'1'; packet_size];
    packet[..prefix.len()].copy_from_slice(prefix.as_bytes());

    loop {
        match socket.send_to(&packet, destination).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send packet: {}", e);
            }
        }
    }
}
