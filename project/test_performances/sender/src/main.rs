// udp_sender.rs
use std::net::{SocketAddr, UdpSocket};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let destination: SocketAddr = "10.42.0.60:8888".parse().expect("Invalid address");
    loop {
        let message = b"$IIHDT,44,T*102";
        match socket.send_to(message, destination) {
            Ok(_) => {}
            Err(_) => {
                println!("ERROR SENDING MESSAGE")
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
