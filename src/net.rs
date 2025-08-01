use iroh::endpoint::{RecvStream, SendStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::Error;

const READ_CHUNK_SIZE: usize = 1_000_000;

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

pub async fn out(
    rx: RecvStream,
    tx: mpsc::UnboundedSender<Option<Vec<u8>>>,
    signal: oneshot::Receiver<()>,
) -> Result<(), Error> {
    let mut rx = rx;
    let mut signal = signal;

    let forwarder = async move {
        let mut buf = vec![0u8; READ_CHUNK_SIZE];
        loop {
            match rx.read(&mut buf).await? {
                None => {
                    tx.send(None).unwrap();
                    break;
                }
                Some(n) => {
                    tx.send(Some(buf[0..n].to_vec())).unwrap();
                }
            }
        }
        Ok::<(), Error>(())
    };

    tokio::select! {
        r = forwarder => {
            r?;
        }
        _ = &mut signal => {
            ()
        }
    }

    Ok(())
}

pub async fn into(
    stream: TcpStream,
    rx: mpsc::UnboundedReceiver<Option<Vec<u8>>>,
    tx: SendStream,
    signal: oneshot::Sender<()>,
) -> Result<(), Error> {
    let mut rx = rx;
    let mut tx = tx;
    let mut stream = stream;
    let mut buf = vec![0u8; READ_CHUNK_SIZE];
    loop {
        let mut active = false;

        match stream.try_read(&mut buf) {
            Ok(n) => {
                if n > 0 {
                    active = true;
                    tx.write_all(&buf[0..n]).await?;
                } else {
                    signal.send(()).unwrap();
                    break;
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => (),
                _ => {
                    Err(e)?;
                }
            },
        }

        if let Ok(data) = rx.try_recv() {
            match data {
                None => break,
                Some(data) => {
                    active = true;
                    stream.write_all(&data).await?;
                }
            }
        }

        if !active {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    tx.finish()?;

    Ok(())
}
