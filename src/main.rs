use clap::Parser;
use random_str::random::{CharBuilder, RandomStringBuilder};
use std::net::SocketAddr;
use tokio::net::TcpListener;

fn random_private_key() -> String {
    RandomStringBuilder::new()
        .with_length(32)
        .with_lowercase()
        .with_numbers()
        .build()
        .unwrap()
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(env, long, default_value = "0.0.0.0:3000")]
    listen_admin: SocketAddr,

    #[clap(env, long, default_value = "0.0.0.0:3001")]
    listen_proxy: SocketAddr,

    #[clap(env, long)]
    konomitv_host: String,

    #[clap(env, long)]
    proxy_host: String,

    #[clap(env, long, default_value_t = random_private_key())]
    private_key: String,
}

mod admin;
mod proxy;
mod sign;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    tracing::info!("Private Key: {}", cli.private_key);

    let admin_sock = TcpListener::bind(cli.listen_admin)
        .await
        .expect("Can be bind admin port");

    tracing::info!("Listen (admin): {}", cli.listen_admin);

    let proxy_sock = TcpListener::bind(cli.listen_proxy)
        .await
        .expect("Can be bind admin port");

    tracing::info!("Listen (proxy): {}", cli.listen_proxy);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 20;

    tracing::info!("{time}");
    let signature = sign::sign(&cli.private_key, "gr011", "480p", time);

    tracing::info!("{signature}");

    tokio::select! {
        _ = proxy::run(proxy_sock, &cli.konomitv_host, &cli.private_key) => {},
        _ = admin::run(admin_sock, &cli.proxy_host, &cli.konomitv_host, &cli.private_key) => {},
    }
}
