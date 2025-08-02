use std::fmt::Debug;

use iroh::{
    NodeId,
    endpoint::{Connection, RecvStream, SendStream},
    protocol::{AcceptError, ProtocolHandler},
};
use tokio::net::TcpStream;

use crate::{AllowList, Error, TunnelRequest, TunnelResponse, net};

pub trait NodeAuth {
    fn allow(&self, node: NodeId) -> impl Future<Output = bool> + Send;
}

pub struct Proxy<A: NodeAuth> {
    auth: A,
    allow_list: AllowList,
    bincode_config: bincode::config::Configuration,
}

impl<A: NodeAuth> Debug for Proxy<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Proxy {{ allow_list: {:?} }}", self.allow_list)?;
        Ok(())
    }
}

impl<A: NodeAuth> Proxy<A> {
    pub fn new(auth: A, allow_list: AllowList) -> Self {
        Self {
            auth,
            allow_list,
            bincode_config: bincode::config::standard(),
        }
    }

    async fn read_request(&self, rx: &mut RecvStream) -> Result<TunnelRequest, Error> {
        let req = net::read_frame(rx).await?;
        let req = bincode::decode_from_slice(&req, self.bincode_config)?.0;
        Ok(req)
    }

    async fn write_response(&self, tx: &mut SendStream, rsp: TunnelResponse) -> Result<(), Error> {
        let frame = bincode::encode_to_vec(&rsp, self.bincode_config)?;
        net::write_frame(tx, frame).await?;
        Ok(())
    }

    async fn connect(&self, req: TunnelRequest) -> Result<TcpStream, Error> {
        let stream =
            TcpStream::connect(format!("{}:{}", req.address.host, req.address.port)).await?;

        Ok(stream)
    }
}

impl<A: NodeAuth + Send + Sync + 'static> ProtocolHandler for Proxy<A> {
    #[tracing::instrument]
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let node_id = connection.remote_node_id()?;
        tracing::info!(node_id = ?node_id, "accept");
        if !self.auth.allow(node_id).await {
            tracing::warn!(node_id = ?node_id, "unauthorized_client_node");
            return Err(AcceptError::NotAllowed {});
        }

        let (mut tx, mut rx) = connection.accept_bi().await?;

        let req = self
            .read_request(&mut rx)
            .await
            .map_err(AcceptError::from_err)?;

        tracing::info!(req = ?req, "tunnel_request");

        if let AllowList::Only(hosts) = &self.allow_list {
            if !hosts.iter().any(|h| h == &req.address.host) {
                tracing::warn!(req = ?req, "rejecting_disallowed_host");
                return Err(AcceptError::NotAllowed {});
            }
        }

        match self.connect(req).await {
            Err(e) => {
                tracing::warn!(err = ?e, "connect_failed");

                self.write_response(&mut tx, TunnelResponse::ConnectFailed)
                    .await
                    .map_err(AcceptError::from_err)?;

                tx.finish()?;
            }
            Ok(stream) => {
                tracing::info!(local_addr = ?stream.local_addr(), "connected");

                self.write_response(&mut tx, TunnelResponse::Connected)
                    .await
                    .map_err(AcceptError::from_err)?;

                net::bridge(stream, rx, tx)
                    .await
                    .map_err(AcceptError::from_err)?;
            }
        }

        connection.closed().await;
        Ok(())
    }
}
