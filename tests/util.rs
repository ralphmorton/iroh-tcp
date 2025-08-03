use iroh::{Endpoint, NodeId, SecretKey, Watcher, protocol::Router};
use iroh_tcp::{ALPN, Client, NodeAuth, Proxy};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[allow(dead_code)]
pub async fn tcp_echo(port: u16) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    loop {
        let mut stream = listener.accept().await.unwrap().0;

        tokio::spawn(async move {
            let mut buf = [0u8; 1_000];
            loop {
                let n = stream.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }

                stream.write(&buf[0..n]).await.unwrap();
            }
        });
    }
}

#[allow(dead_code)]
pub async fn http_hello(port: u16) {
    let app = axum::Router::new().route("/", axum::routing::get(|| async { "hello world" }));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

struct TestAuth {
    client: NodeId,
    allow: Allow,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Allow {
    All,
    Specific(Vec<(String, u16)>),
}

impl NodeAuth for TestAuth {
    async fn allow(&self, node: NodeId, host: &str, port: u16) -> bool {
        if node != self.client {
            return false;
        }

        match &self.allow {
            Allow::All => true,
            Allow::Specific(hosts) => hosts.iter().any(|(h, p)| h == host && *p == port),
        }
    }
}

#[allow(dead_code)]
pub struct ClientServer {
    pub client: Client,
    pub client_sk: SecretKey,
    pub server: Router,
    pub server_sk: SecretKey,
}

impl ClientServer {
    pub async fn new(allow: Allow) -> Self {
        let mut rng = rand::thread_rng();
        let server_sk = SecretKey::generate(&mut rng);
        let client_sk = SecretKey::generate(&mut rng);

        let server_endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(server_sk.clone())
            .bind()
            .await
            .unwrap();

        let auth = TestAuth {
            client: client_sk.public(),
            allow,
        };

        let server = Router::builder(server_endpoint)
            .accept(ALPN, Proxy::new(auth))
            .spawn();

        let server_addr = server.endpoint().node_addr().initialized().await;

        let client_endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(client_sk.clone())
            .bind()
            .await
            .unwrap();

        let client = Client::with_addr(client_endpoint, server_addr);

        Self {
            client,
            client_sk,
            server,
            server_sk,
        }
    }
}
