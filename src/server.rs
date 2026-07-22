use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::handlers;


pub async fn start_server() {
    let app = Router::new()
        .route("/", get(root))
        .route("/verb_harmony/{verb}", get(handlers::vowel_harmony::classify));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await;

}

async fn root() -> &'static str {
    "Hello, World!"
}
