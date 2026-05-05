use std::path::Path;

use axum::response::IntoResponse;

fn strip_base<'a>(path: &'a str, base: &str) -> &'a str {
    if base.is_empty() {
        return path;
    }
    let base = base.trim_end_matches('/');
    if path == base {
        "/"
    } else if path.starts_with(base) {
        &path[base.len()..]
    } else {
        path
    }
}

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



pub fn serve(port: u16, base: &str) {
    let dist_path = Path::new("dist");
    if !dist_path.exists() {
        eprintln!("Error: dist/ directory not found. Run `ravel --build <file>` first.");
        std::process::exit(1);
    }
    if !dist_path.is_dir() {
        eprintln!("Error: dist/ is not a directory.");
        std::process::exit(1);
    }

    let base = base.trim_end_matches('/').to_string();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let app = axum::Router::new().fallback_service(
            tower_http::services::ServeDir::new("dist")
                .fallback(axum::handler::HandlerWithoutStateExt::into_service(html_fallback)),
        );

        let app = if base.is_empty() {
            app
        } else {
            let b = base.clone();
            app.layer(axum::middleware::from_fn(
                move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                    let b = b.clone();
                    async move {
                        let original_path = req.uri().path().to_string();
                        let stripped = strip_base(&original_path, &b);
                        if stripped != original_path {
                            let mut parts = req.uri().clone().into_parts();
                            let pq = format!(
                                "{}{}",
                                stripped,
                                parts.path_and_query.as_ref().and_then(|pq| pq.query()).map(|q| format!("?{}", q)).unwrap_or_default()
                            );
                            parts.path_and_query = Some(pq.parse().unwrap());
                            *req.uri_mut() = axum::http::Uri::from_parts(parts).unwrap();
                        }
                        next.run(req).await
                    }
                },
            ))
        };

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let display_base = if base.is_empty() {
            "/".to_string()
        } else {
            base.clone()
        };
        println!("Serving dist/ at http://{}{}", addr, display_base);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");
        axum::serve(listener, app)
            .await
            .expect("Server error");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_base_no_base() {
        assert_eq!(strip_base("/style.css", ""), "/style.css");
        assert_eq!(strip_base("/", ""), "/");
    }

    #[test]
    fn test_strip_base_with_base() {
        assert_eq!(strip_base("/repo/style.css", "/repo"), "/style.css");
        assert_eq!(strip_base("/repo", "/repo"), "/");
        assert_eq!(strip_base("/repo/", "/repo"), "/");
    }

    #[test]
    fn test_strip_base_no_match() {
        assert_eq!(strip_base("/other/style.css", "/repo"), "/other/style.css");
    }

    #[test]
    fn test_strip_base_trailing_slash() {
        assert_eq!(strip_base("/repo/style.css", "/repo/"), "/style.css");
    }
}