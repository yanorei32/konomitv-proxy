use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::StatusCode;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    secret: String,
    konomitv_host: String,
    proxy_host: String,
}

pub async fn root() -> impl IntoResponse {
    (
        [(http::header::CONTENT_TYPE, "text/html")],
        include_str!("../assets/index.html"),
    )
}

async fn channels(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let upstream_url = format!("{}/api/channels", state.konomitv_host);

    tracing::info!("Requesting to {upstream_url}");

    let request = state.client.get(upstream_url);

    let upstream = request.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let status = upstream.status();
    let headers = upstream.headers().clone();

    let body = Body::from_stream(upstream.bytes_stream());

    let mut response = Response::new(body);
    *response.status_mut() = status;

    for (name, value) in &headers {
        if name != "connection" && name != "transfer-encoding" {
            response.headers_mut().insert(name.clone(), value.clone());
        }
    }

    Ok(response)
}

async fn auth(
    State(state): State<AppState>,
    Path((channel, resolution, duration)): Path<(String, String, u64)>,
) -> String {
    let unixtime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before the UNIX epoch");

    let timelimit = unixtime.as_secs() + duration;

    format!(
        "{}/{channel}/{resolution}/{timelimit}/{}",
        state.proxy_host,
        crate::sign::sign(&state.secret, &channel, &resolution, timelimit)
    )
}

pub async fn run(listener: TcpListener, proxy_host: &str, konomitv_host: &str, secret: &str) {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/channels", get(channels))
        .route("/api/auth/{channel}/{resolution}/{duration}", get(auth))
        .with_state(AppState {
            client: reqwest::Client::new(),
            secret: secret.to_string(),
            konomitv_host: konomitv_host.to_string(),
            proxy_host: proxy_host.to_string(),
        });

    axum::serve(listener, app).await.unwrap();
}
