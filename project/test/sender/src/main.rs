use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn main() {
    // Define the target address (replace with your desired IP and port)
    let target_addr: SocketAddr = "10.42.0.60:8888".parse().expect("Invalid address");

    // Create a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    // Set the target bitrate to 1 Gbit/s (125 MB/s)
    let target_bitrate_bps: u64 = 1_000_000_000; // 1 Gbit/s
    let target_bytes_per_second = target_bitrate_bps / 8;

    // Define the payload size (e.g., 1400 bytes per packet)
    let payload_size: usize = 1460;
    let header = "$IIHDT,33,T*44";
    let mut buffer = vec![0u8; payload_size];
    buffer[..header.len()].copy_from_slice(header.as_bytes());

    // Calculate the number of packets per second
    let packets_per_second = target_bytes_per_second / payload_size as u64;

    // Calculate the interval between packets (in nanoseconds)
    let interval_ns = 1_000_000_000 / packets_per_second;

    println!(
        "Sending {} packets per second, each of size {} bytes, interval: {} ns",
        packets_per_second, payload_size, interval_ns
    );

    // Start sending packets

    loop {
        match socket.send_to(&buffer, target_addr) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send packet: {}", e);
            }
        }

        thread::sleep(Duration::from_nanos(interval_ns));
    }
}
