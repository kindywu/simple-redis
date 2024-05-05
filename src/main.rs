use std::net::SocketAddr;

use anyhow::Result;
use futures::SinkExt;
use tokio_stream::StreamExt;

use simple_redis::{Command, CommandExecutor, RespDecode, RespEncode, RespError, RespFrame};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis-Server is listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        tokio::spawn(async move {
            match process_redis_conn(stream, raddr).await {
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

async fn process_redis_conn(stream: TcpStream, _: SocketAddr) -> Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let request = RedisRequest { frame };
                let response = request_handler(request).await?;
                info!("Sending response: {:?}", response.frame);
                framed.send(response.frame).await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}

async fn request_handler(request: RedisRequest) -> Result<RedisResponse> {
    let frame = request.frame;
    let cmd = Command::try_from(frame)?;
    info!("Executing command: {:?}", cmd);
    let frame = cmd.execute();
    Ok(RedisResponse { frame })
}

#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
}

#[derive(Debug)]
struct RedisResponse {
    frame: RespFrame,
}

#[derive(Debug)]
struct RespFrameCodec;

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<RespFrame>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
