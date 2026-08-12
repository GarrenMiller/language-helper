use axum::{
    routing::get,
    Router,
};

use crate::handlers;

// Macro to allow any function on a route handler for debugging
macro_rules! debug_handler {
    ($expr:expr) => {
        || async {
            match $expr {
                Ok(_) => (),
                Err(e) => eprintln!("debug handler error: {e}")
            }
            StatusCode::OK
        }
    };
}

pub async fn start_server() {
    let app = Router::new()
        .route("/", get(root))
        .route("/fst", get(handlers::morphology::load_analyzer_binary))
        .route("/verb_harmony/{verb}", get(handlers::vowel_harmony::classify));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    eprintln!("Shutting down server...");
}

async fn root() -> &'static str {
    "Hello, World!"
}
