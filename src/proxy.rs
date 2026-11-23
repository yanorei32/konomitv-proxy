use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::get,
};
use futures_util::StreamExt;
use reqwest::StatusCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{net::TcpListener, time::Instant};

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    secret: String,
    konomitv_host: String,
}

async fn stream(
    Path((channel, resolution, unixtime, signature)): Path<(String, String, u64, String)>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let upstream_url = format!(
        "{}/api/streams/live/{channel}/{resolution}/mpegts",
        state.konomitv_host
    );

    if !crate::sign::verify(&state.secret, &channel, &resolution, unixtime, &signature) {
        tracing::info!("Invalid Signature");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let limit = UNIX_EPOCH + Duration::from_secs(unixtime);

    let Ok(limit) = limit.duration_since(SystemTime::now()) else {
        tracing::info!("Expired");
        return Err(StatusCode::UNAUTHORIZED);
    };

    let limit = Instant::now() + limit;

    tracing::info!("Requesting to {upstream_url}");

    let request = state.client.get(upstream_url);

    let upstream = request.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let status = upstream.status();
    let headers = upstream.headers().clone();

    let mut upstream_stream = upstream.bytes_stream();

    let limited_stream = async_stream::stream! {
        loop {
            tokio::select! {
                v = upstream_stream.next() => {
                    match v {
                        Some(Ok(chunk)) => yield Ok(chunk),
                        Some(Err(e)) => {
                            yield Err(e);
                            break;
                        }
                        None => break,
                    }
                }
                _ = tokio::time::sleep_until(limit) => break,
            }
        }
    };

    let body = Body::from_stream(limited_stream);

    let mut response = Response::new(body);
    *response.status_mut() = status;

    for (name, value) in &headers {
        if name != "connection" && name != "transfer-encoding" {
            response.headers_mut().insert(name.clone(), value.clone());
        }
    }

    Ok(response)
}

pub async fn run(listener: TcpListener, konomitv_host: &str, secret: &str) {
    let app = Router::new()
        .route(
            "/{channel}/{resolution}/{unixtime}/{signature}",
            get(stream),
        )
        .with_state(AppState {
            client: reqwest::Client::new(),
            secret: secret.to_string(),
            konomitv_host: konomitv_host.to_string(),
        });

    axum::serve(listener, app).await.unwrap();
}
