use bincode::{Decode, Encode};

pub const ALPN: &[u8] = b"iroh-echo";

pub type Host = String;

#[derive(Clone, Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct Address {
    pub host: Host,
    pub port: u16,
}

#[derive(Debug, Decode, Encode)]
pub struct TunnelRequest {
    pub address: Address,
}

#[derive(Debug, Decode, Encode)]
pub enum TunnelResponse {
    Connected,
    ConnectFailed,
}

#[derive(Clone, Debug)]
pub enum AllowList {
    All,
    Only(Vec<Host>),
}
