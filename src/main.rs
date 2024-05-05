use std::net::SocketAddr;

use anyhow::Result;
use futures::SinkExt;
use tokio_stream::StreamExt;

use simple_redis::{Command, CommandExecutor, RespFrameCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use tracing::{info, warn};

const ADDR: &str = "0.0.0.0:6379";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Simple-Redis-Server is listening on {}", ADDR);
    let listener = TcpListener::bind(ADDR).await?;

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        tokio::spawn(async move {
            match process_conn(stream, raddr).await {
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

async fn process_conn(stream: TcpStream, _: SocketAddr) -> Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);

                let cmd = Command::try_from(frame)?;
                info!("Executing command: {:?}", cmd);

                let frame = cmd.execute();
                info!("Sending response: {:?}", frame);
                framed.send(frame).await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}
