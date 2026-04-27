use axum::body::Body;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use std::sync::Arc;

use crate::audio::AudioProcessor;

mod lufs;
mod audio;

#[derive(Parser)]
struct Args {
    #[arg(short('b'), long("bind"), default_value_t={"0.0.0.0:9011".to_string()})]
    /// The endpoint to bind onto
    bind_address: String,
    #[arg(short('d'), long("device"))]
    /// The device id for the microphone to monitor. Defaults to the system's default input.
    input_device: Option<String>
}

pub async fn metrics_handler(State(processor): State<Arc<AudioProcessor>>) -> impl IntoResponse {
    let snapshot = processor.snapshot();
    let momentary = snapshot.momentary();
    let short_term = snapshot.short_term();

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(Body::from(format!(
"# HELP sound_momentary The momentary LUFS from the server
# TYPE sound_momentary gauge
sound_momentary {momentary}
# HELP sound_short_term The short-term LUFS from the server
# TYPE sound_short_term gauge
sound_short_term {short_term}
"
        )))
        .unwrap()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let processor = AudioProcessor::new(args.input_device);
    let router = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(Arc::new(processor));
    let listener = tokio::net::TcpListener::bind(args.bind_address)
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}
