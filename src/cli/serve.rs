use axum::{
    Router,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
};
use percent_encoding::percent_decode_str;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct ServerState {
    output_dir: Arc<PathBuf>,
}

async fn html_path_resolver(
    State(state): State<ServerState>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    let decoded_path = percent_decode_str(path).decode_utf8_lossy();

    let has_extension = decoded_path
        .split('/')
        .last()
        .map_or(false, |p| p.contains('.'));
    let is_directory = decoded_path.ends_with('/');

    if !has_extension && !is_directory {
        let html_file_path = state
            .output_dir
            .join(decoded_path.trim_start_matches('/'))
            .with_extension("html");

        if html_file_path.exists() {
            let new_uri = format!("{}.html", req.uri());
            if let Ok(uri) = new_uri.parse() {
                *req.uri_mut() = uri;
            }
        }
    }

    next.run(req).await
}

pub async fn start_server(output_dir: &PathBuf) -> Result<(), std::io::Error> {
    let state = ServerState {
        output_dir: Arc::new(output_dir.to_path_buf()),
    };

    let app = Router::new()
        .fallback_service(ServeDir::new(output_dir))
        .layer(middleware::from_fn_with_state(state, html_path_resolver));

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;

    println!("🚀 Server running at http://localhost:8080");

    axum::serve(listener, app).await
}
