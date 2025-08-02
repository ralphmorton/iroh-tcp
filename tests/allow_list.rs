use iroh_tcp::{Address, AllowList};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use util::{ClientServer, tcp_echo};

mod util;

#[tokio::test]
async fn allow_list() {
    let allow_list = AllowList::Only(vec!["127.0.0.1".to_string()]);
    let _echo = tokio::spawn(tcp_echo(3002));

    let client_server = ClientServer::new(allow_list.clone()).await;
    let tunnel = tokio::spawn(async move {
        client_server
            .client
            .clone()
            .listen(
                "localhost",
                3001,
                Address {
                    host: "127.0.0.1".to_string(),
                    port: 3002,
                },
            )
            .await
            .unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut stream = TcpStream::connect("localhost:3001").await.unwrap();
    stream.write(b"test").await.unwrap();
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"test");

    tunnel.abort();

    let client_server = ClientServer::new(allow_list).await;
    let tunnel = tokio::spawn(async move {
        client_server
            .client
            .clone()
            .listen(
                "localhost",
                3001,
                Address {
                    host: "localhost".to_string(),
                    port: 3002,
                },
            )
            .await
            .unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut stream = TcpStream::connect("localhost:3001").await.unwrap();
    stream.write(b"test").await.unwrap();
    let mut buf = [0u8; 4];
    let read_res = stream.read_exact(&mut buf).await;

    assert!(matches!(read_res, Err(_)));

    let read_err = read_res.err().unwrap();
    assert_eq!(read_err.kind(), std::io::ErrorKind::ConnectionReset);

    tunnel.abort();
}
