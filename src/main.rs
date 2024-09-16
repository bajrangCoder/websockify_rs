use futures_util::{SinkExt, StreamExt};
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <source_addr:port> <target_addr:port>", args[0]);
        std::process::exit(1);
    }

    let source_addr = &args[1];
    let target_addr = &args[2];

    println!("WebSocket settings:");
    println!("    - proxying from {} to {}", source_addr, target_addr);

    let listener = TcpListener::bind(source_addr).await?;
    println!("Listening on: {}", source_addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let target_addr = target_addr.to_string();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, &target_addr).await {
                eprintln!("Error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    target_addr: &str,
) -> Result<(), Box<dyn Error>> {
    let ws_stream = accept_async(stream).await?;
    println!("WebSocket connection established");

    let start_time = get_timestamp();

    let target_stream = TcpStream::connect(target_addr).await?;
    println!("Connected to target: {}", target_addr);

    let (ws_sender, ws_receiver) = ws_stream.split();
    let (target_reader, target_writer) = target_stream.into_split();

    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));
    let target_writer = Arc::new(tokio::sync::Mutex::new(target_writer));

    let to_target = tokio::spawn({
        let target_writer = Arc::clone(&target_writer);
        async move {
            let mut ws_receiver = ws_receiver;
            while let Some(message) = ws_receiver.next().await {
                let message = message?;
                if message.is_binary() || message.is_text() {
                    let data = message.into_data();
                    let mut writer = target_writer.lock().await;
                    writer.write_all(&data).await?;
                }
            }
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        }
    });

    let from_target = tokio::spawn({
        let ws_sender = Arc::clone(&ws_sender);
        async move {
            let mut target_reader = target_reader;
            let mut buffer = [0; 1024];
            loop {
                let n = target_reader.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                let mut sender = ws_sender.lock().await;
                sender.send(Message::Binary(buffer[..n].to_vec())).await?;
            }
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        }
    });

    tokio::try_join!(to_target, from_target)?;

    let duration = get_timestamp() - start_time;
    println!(
        "WebSocket client {} is disconnected after {} ms",
        addr, duration
    );

    Ok(())
}
