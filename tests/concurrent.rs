mod util;

use iroh_tcp::Address;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use util::{Allow, ClientServer, tcp_echo};

#[tokio::test]
async fn echo_concurrent() {
    let client_server = ClientServer::new(Allow::All).await;
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

    let run = |msg: String| async move {
        let mut stream = TcpStream::connect("localhost:3001").await.unwrap();
        let mut echos = vec![];

        for i in 1..5 {
            let msg = format!("{msg} {i}");
            stream.write(msg.as_bytes()).await.unwrap();

            let mut buf = vec![0u8; msg.len()];
            stream.read_exact(&mut buf).await.unwrap();

            echos.push(buf);
        }

        stream.shutdown().await.unwrap();
        echos
    };

    let tasks = (1..5).map(|i| {
        let name = format!("echo{i}");
        let task = tokio::spawn(run(name.clone()));
        (name, task)
    });

    for (name, task) in tasks.into_iter() {
        let res = task.await.unwrap();

        let expected = vec![
            format!("{} {}", name, 1).as_bytes().to_vec(),
            format!("{} {}", name, 2).as_bytes().to_vec(),
            format!("{} {}", name, 3).as_bytes().to_vec(),
            format!("{} {}", name, 4).as_bytes().to_vec(),
        ];

        assert_eq!(res, expected);
    }

    tunnel.abort();
}
