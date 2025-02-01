use std::net::UdpSocket;
use std::time::{Duration, Instant};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8080").expect("Failed to bind socket");
    println!("Listening on 0.0.0.0:8080");

    let mut buf = [0; 65535];

    let start_time = Instant::now();
    let duration = Duration::from_secs(10);
    let mut total_bytes_received: usize = 0;

    while Instant::now() - start_time < duration {
        match socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                total_bytes_received += size;
            }
            Err(e) => eprintln!("Failed to receive message: {}", e),
        }
    }

    let elapsed_time = start_time.elapsed().as_secs_f64();
    let bitrate = (total_bytes_received as f64 * 8.0) / elapsed_time;

    println!(
        "Total bytes received: {}, Time elapsed: {:.2} seconds, Bitrate: {:.2} bps",
        total_bytes_received, elapsed_time, bitrate
    );
}
