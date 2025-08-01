#[derive(Debug)]
pub enum Error {
    ConnectionError(iroh::endpoint::ConnectionError),
    ConnectError(iroh::endpoint::ConnectError),
    CloseError(iroh::endpoint::ClosedStream),
    ReadError(iroh::endpoint::ReadError),
    ReadExactError(iroh::endpoint::ReadExactError),
    WriteError(iroh::endpoint::WriteError),
    KeyParsingError(iroh::KeyParsingError),
    DecodeError(bincode::error::DecodeError),
    EncodeError(bincode::error::EncodeError),
    IoError(std::io::Error),
    TunnelConnectionFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionError(e) => write!(f, "ConnectionError: {:?}", e),
            Self::ConnectError(e) => write!(f, "ConnectError: {:?}", e),
            Self::CloseError(e) => write!(f, "CloseError: {:?}", e),
            Self::ReadError(e) => write!(f, "ReadError: {:?}", e),
            Self::ReadExactError(e) => write!(f, "ReadExactError: {:?}", e),
            Self::WriteError(e) => write!(f, "WriteError: {:?}", e),
            Self::KeyParsingError(e) => write!(f, "KeyParsingError: {:?}", e),
            Self::DecodeError(e) => write!(f, "DecodeError: {:?}", e),
            Self::EncodeError(e) => write!(f, "EncodeError: {:?}", e),
            Self::IoError(e) => write!(f, "IoError: {:?}", e),
            Self::TunnelConnectionFailed => write!(f, "TunnelConnectionFailed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<iroh::endpoint::ConnectionError> for Error {
    fn from(value: iroh::endpoint::ConnectionError) -> Self {
        Self::ConnectionError(value)
    }
}

impl From<iroh::endpoint::ConnectError> for Error {
    fn from(value: iroh::endpoint::ConnectError) -> Self {
        Self::ConnectError(value)
    }
}

impl From<iroh::endpoint::ClosedStream> for Error {
    fn from(value: iroh::endpoint::ClosedStream) -> Self {
        Self::CloseError(value)
    }
}

impl From<iroh::endpoint::ReadError> for Error {
    fn from(value: iroh::endpoint::ReadError) -> Self {
        Self::ReadError(value)
    }
}

impl From<iroh::endpoint::ReadExactError> for Error {
    fn from(value: iroh::endpoint::ReadExactError) -> Self {
        Self::ReadExactError(value)
    }
}

impl From<iroh::endpoint::WriteError> for Error {
    fn from(value: iroh::endpoint::WriteError) -> Self {
        Self::WriteError(value)
    }
}

impl From<iroh::KeyParsingError> for Error {
    fn from(value: iroh::KeyParsingError) -> Self {
        Self::KeyParsingError(value)
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::DecodeError(value)
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::EncodeError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
