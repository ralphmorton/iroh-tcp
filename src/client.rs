use std::fmt::Debug;

use iroh::{
    Endpoint, NodeAddr, NodeId,
    endpoint::{RecvStream, SendStream},
};
use tokio::net::{TcpListener, TcpStream};

use crate::{ALPN, Address, Either, Error, TunnelRequest, TunnelResponse, net};

#[derive(Clone)]
pub struct Client {
    endpoint: Endpoint,
    server: Either<NodeAddr, NodeId>,
    bincode_config: bincode::config::Configuration,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Client {{ endpoint: {:?}, server: {:?} }}",
            self.endpoint, self.server
        )?;
        Ok(())
    }
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

    #[tracing::instrument]
    pub async fn listen(&self, host: &str, port: u16, address: Address) -> Result<(), Error> {
        tracing::info!(host = host, port = port, "listen");
        let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
        tracing::info!("listening");

        loop {
            let stream = listener.accept().await?.0;
            tracing::info!(local_addr = ?stream.local_addr(), "client_connected");

            let c = self.clone();
            let address = address.clone();
            tokio::spawn(async move {
                tracing::info!("starting_tunnel");
                if let Err(e) = c.tunnel(stream, address).await {
                    tracing::warn!(err = ?e, "tunnel_error");
                }
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

        net::bridge(stream, rx, tx).await?;
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
