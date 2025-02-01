use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

fn main() {
    let message_size: usize = 8972;
    let mut message: Vec<u8> = vec![b'A'; message_size];

    message[0] = b'$';
    message[1] = b'I';
    message[2] = b'I';
    message[3] = b'H';
    message[4] = b'D';
    message[5] = b'T';
    message[6] = b',';
    message[7] = b'3';
    message[8] = b'3';
    message[9] = b',';
    message[10] = b'T';
    message[11] = b'*';

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let destination: SocketAddr = "10.42.0.60:8888".parse().expect("Invalid address");

    let start_time = Instant::now();
    let duration = Duration::from_secs(10); // Run for 10 seconds
    let mut total_bytes_sent: usize = 0;

    while Instant::now() - start_time < duration {
        match socket.send_to(&message, destination) {
            Ok(bytes_sent) => {
                total_bytes_sent += bytes_sent;
            }
            Err(e) => eprintln!("Failed to send message: {}", e),
        }
        std::thread::sleep(std::time::Duration::from_millis(1)); // Small delay to avoid flooding
    }

    let elapsed_time = start_time.elapsed().as_secs_f64();
    let bitrate = (total_bytes_sent as f64 * 8.0) / elapsed_time; // Convert bytes to bits and calculate bitrate

    println!(
        "Total bytes sent: {}, Time elapsed: {:.2} seconds, Bitrate: {:.2} bps",
        total_bytes_sent, elapsed_time, bitrate
    );
}
