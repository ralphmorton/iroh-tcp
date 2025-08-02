use iroh::endpoint::{RecvStream, SendStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
};

use crate::Error;

pub async fn read_frame(rx: &mut RecvStream) -> Result<Vec<u8>, Error> {
    let frame_size = rx.read_u16_le().await?;
    let mut frame = vec![0u8; frame_size as usize];
    rx.read_exact(&mut frame).await?;
    Ok(frame)
}

pub async fn write_frame(tx: &mut SendStream, frame: Vec<u8>) -> Result<(), Error> {
    tx.write_u16_le(frame.len() as u16).await?;
    tx.write_all(&frame).await?;
    tx.flush().await?;
    Ok(())
}

pub async fn bridge(
    stream: TcpStream,
    mut rx: RecvStream,
    mut tx: SendStream,
) -> Result<(), Error> {
    let (mut stream_rx, mut stream_tx) = tokio::io::split(stream);

    let out: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        tokio::io::copy(&mut rx, &mut stream_tx).await?;
        Ok(())
    });

    tokio::io::copy(&mut stream_rx, &mut tx).await?;
    tx.finish()?;
    out.await??;

    Ok(())
}
