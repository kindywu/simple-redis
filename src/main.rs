use std::net::SocketAddr;

use anyhow::Result;
use futures::SinkExt;
use tokio_stream::StreamExt;

use simple_redis::{Backend, Command, CommandExecutor, RespEncode, RespFrameCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use tracing::{info, warn};

const ADDR: &str = "0.0.0.0:6379";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let backend = Backend::new();

    info!("Simple-Redis-Server is listening on {}", ADDR);
    let listener = TcpListener::bind(ADDR).await?;

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        let backend = backend.clone();
        tokio::spawn(async move {
            // println!(":?", backend);
            match process_conn(stream, raddr, backend).await {
                Ok(_) => {
                    info!("Connection from {} exited", raddr);
                }
                Err(e) => {
                    warn!("handle error for {}: {:?}", raddr, e);
                }
            }
        });
    }
}

async fn process_conn(stream: TcpStream, _: SocketAddr, backend: Backend) -> Result<()> {
    println!("{:?}", backend);
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                info!(
                    "Received frame: {:?}",
                    String::from_utf8(frame.clone().encode())
                );

                let cmd = Command::try_from(frame)?;
                info!("Executing command: {:?}", cmd);

                let frame = cmd.execute(&backend);
                info!("Sending response: {:?}", frame);
                framed.send(frame).await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}
