mod util;

use iroh_tcp::{Address, AllowList};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use util::{ClientServer, tcp_echo};

#[tokio::test]
async fn echo() {
    let client_server = ClientServer::new(AllowList::All).await;
    let _echo = tokio::spawn(tcp_echo(3002));

    let tunnel = tokio::spawn(async move {
        client_server
            .client
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
    for i in 0..5 {
        let msg = format!("msg {i}").as_bytes().to_vec();
        stream.write(&msg).await.unwrap();

        let mut buf = [0u8; 5];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, msg[0..5]);
    }
    stream.shutdown().await.unwrap();

    tunnel.abort();
}
