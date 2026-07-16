use agent_passport_api::router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let port = std::env::var("AGENT_PASSPORT_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8787);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind local offchain API");

    println!("Agent Passport API listening on http://{addr}");
    println!("Network: offchain-devnet; live chain: false");

    axum::serve(listener, router()).await.expect("serve API");
}
