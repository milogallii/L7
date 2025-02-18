use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn main() {
    let target_addr: SocketAddr = "10.42.0.60:8888".parse().expect("Invalid address");
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    //prepare NMEA message
    let payload_size: usize = 1460;
    let header = "$IIHDT,33,T*44";
    let mut buffer = vec![0u8; payload_size];
    buffer[..header.len()].copy_from_slice(header.as_bytes());

    loop {
        match socket.send_to(&buffer, target_addr) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send packet: {}", e);
            }
        }

        thread::sleep(Duration::from_nanos(1));
    }
}
