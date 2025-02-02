use std::fs::OpenOptions;
use std::io::{Write, self};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::net::UdpSocket;
use anyhow::{Result, Error};
use uuid::Uuid;
use std::fs::File;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let socket = UdpSocket::bind("0.0.0.0:8888").await?;
    println!("Listening for UDP packets on 0.0.0.0:8888...");
    
    let mut buf = [0; 65535];
    let mut total_bytes_received: usize = 0;
    let id = Uuid::new_v4();
    let filename = format!("{}.log", id);
    
    // Wrap the file in an Arc<Mutex<>> so it can be shared across tasks
    let file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)?
    ));
    
    let mut interval_start = Instant::now();
    
    loop {
        let (bytes_received, _) = socket.recv_from(&mut buf).await?;
        total_bytes_received += bytes_received;
        
        if interval_start.elapsed() >= Duration::from_secs(10) {
            let file_clone = Arc::clone(&file);
            
            tokio::spawn(async move {
                if let Err(e) = log_udp(file_clone, total_bytes_received.clone()).await {
                    eprintln!("Failed to log UDP data: {}", e);
                }
            });
            
            total_bytes_received = 0;
            interval_start = Instant::now();
        }
    }
}

async fn log_udp(file: Arc<Mutex<File>>, total_bytes_received: usize) -> Result<(), Error> {
    let mut file = file.lock().await; // Lock the file for writing
    writeln!(
        file,
        "[{:?}] BYTES RECEIVED: {}",
        SystemTime::now(),
        total_bytes_received
    )?;
    file.flush()?; 
    Ok(())
}
