mod util;

use iroh_tcp::Address;
use util::{ClientServer, http_hello};

#[tokio::test]
async fn http_basic() {
    let client_server = ClientServer::new().await;
    let _http = tokio::spawn(http_hello(3002));

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

    let rsp = reqwest::get("http://localhost:3001")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(rsp, "hello world");

    tunnel.abort();
}
