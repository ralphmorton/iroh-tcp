use iroh::{
    Endpoint, NodeAddr, NodeId,
    endpoint::{RecvStream, SendStream},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

use crate::{ALPN, Address, Either, Error, TunnelRequest, TunnelResponse, net};

#[derive(Clone)]
pub struct Client {
    endpoint: Endpoint,
    server: Either<NodeAddr, NodeId>,
    bincode_config: bincode::config::Configuration,
}

impl Client {
    pub fn new(endpoint: Endpoint, server: NodeId) -> Self {
        Self {
            endpoint,
            server: Either::Right(server),
            bincode_config: bincode::config::standard(),
        }
    }

    pub fn with_addr(endpoint: Endpoint, server: NodeAddr) -> Self {
        Self {
            endpoint,
            server: Either::Left(server),
            bincode_config: bincode::config::standard(),
        }
    }

    pub async fn listen(&self, host: &str, port: u16, address: Address) -> Result<(), Error> {
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
        loop {
            let stream = listener.accept().await?.0;

            let c = self.clone();
            let address = address.clone();
            tokio::spawn(async move {
                let _ = c.tunnel(stream, address).await;
            });
        }
    }

    pub async fn tunnel(&self, stream: TcpStream, address: Address) -> Result<(), Error> {
        let conn = match &self.server {
            Either::Left(node_id) => self.endpoint.connect(node_id.clone(), ALPN).await?,
            Either::Right(node_addr) => self.endpoint.connect(node_addr.clone(), ALPN).await?,
        };

        let (mut tx, mut rx) = conn.open_bi().await?;

        self.write_request(&mut tx, TunnelRequest { address })
            .await?;

        if let TunnelResponse::ConnectFailed = self.read_response(&mut rx).await? {
            return Err(Error::TunnelConnectionFailed);
        }

        let (data_tx, data_rx) = mpsc::unbounded_channel();
        let (signal_tx, signal_rx) = oneshot::channel();

        let out = tokio::spawn(net::out(rx, data_tx, signal_rx));
        let into = tokio::spawn(net::into(stream, data_rx, tx, signal_tx));

        out.await.unwrap()?;
        into.await.unwrap()?;

        conn.close(0u32.into(), b"bye");
        Ok(())
    }

    async fn write_request(&self, tx: &mut SendStream, req: TunnelRequest) -> Result<(), Error> {
        let frame = bincode::encode_to_vec(&req, self.bincode_config)?;
        net::write_frame(tx, frame).await?;
        Ok(())
    }

    async fn read_response(&self, rx: &mut RecvStream) -> Result<TunnelResponse, Error> {
        let frame = net::read_frame(rx).await?;
        let rsp = bincode::decode_from_slice(&frame, self.bincode_config)?.0;
        Ok(rsp)
    }
}
