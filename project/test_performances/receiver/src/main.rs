use std::net::UdpSocket;
// use std::time::{Duration, Instant};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8888").expect("Failed to bind socket");
    println!("Listening on 0.0.0.0:8888");

    let mut buf = [0; 1024];
    let mut message_count = 0;
    let mut total_volume = 0;
    // let start_time = Instant::now();
    // let duration = Duration::from_secs(10); // Run for 10 seconds

    // while Instant::now() - start_time < duration {
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                // message_count += 1;
                // total_volume += size;
                println!("received a message");
            }
            Err(e) => eprintln!("Failed to receive message: {}", e),
        }
    }

    println!("{} - {} ", message_count, total_volume);
}
