use iroh::{Endpoint, SecretKey, Watcher};
use iroh_tcp::{Address, AllowList, Client};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use util::{ClientServer, tcp_echo};

mod util;

#[tokio::test]
async fn rejects_unauthorized_client() {
    let client_server = ClientServer::new(AllowList::All).await;
    let _echo = tokio::spawn(tcp_echo(3002));

    let mut rng = rand::thread_rng();
    let client_sk = SecretKey::generate(&mut rng);
    let client_endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(client_sk.clone())
        .bind()
        .await
        .unwrap();

    let server_addr = client_server
        .server
        .endpoint()
        .node_addr()
        .initialized()
        .await;

    let client = Client::with_addr(client_endpoint, server_addr);

    let tunnel = tokio::spawn(async move {
        client
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
