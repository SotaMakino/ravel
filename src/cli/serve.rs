use std::path::Path;

use axum::handler::HandlerWithoutStateExt;
use axum::response::IntoResponse;

async fn html_fallback(req: axum::extract::Request) -> axum::response::Response {
    let path = req.uri().path().trim_start_matches('/');
    if path.contains("..") {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    let html_path = format!("dist/{}.html", path);

    if path.is_empty() {
        return match tokio::fs::read("dist/index.html").await {
            Ok(bytes) => axum::response::Html(bytes).into_response(),
            Err(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
        };
    }

    if let Ok(bytes) = tokio::fs::read(&html_path).await {
        return axum::response::Html(bytes).into_response();
    }

    axum::http::StatusCode::NOT_FOUND.into_response()
}

pub fn serve(port: u16) {
    let dist_path = Path::new("dist");
    if !dist_path.exists() {
        eprintln!("Error: dist/ directory not found. Run `ravel --build <file>` first.");
        std::process::exit(1);
    }
    if !dist_path.is_dir() {
        eprintln!("Error: dist/ is not a directory.");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let app = axum::Router::new().fallback_service(
            tower_http::services::ServeDir::new("dist")
                .fallback(html_fallback.into_service()),
        );

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        println!("Serving dist/ at http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");
        axum::serve(listener, app)
            .await
            .expect("Server error");
    });
}

