use axum::{
    Json, Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use serde_json::{Value, json};
use http_body_util::BodyExt;
use tower_http::services::ServeDir;

const WEB_ROOT: &str = "/app/web";
const LAYOUT_FILE: &str = "/app/web/.layout.html";

#[derive(Debug)]
enum ApiError {
    NotFound,
    InvalidInput(String),
    InternalError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response{
        let (status, error_message) = match self{
            ApiError::NotFound => (
                StatusCode::NOT_FOUND, "Data not found".to_string(),
            ),
            ApiError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST, msg,
            ),
            ApiError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Server is running"
    }))
}

/// Reads `.layout.html`, reads the page fragment, substitutes `{{outlet}}`, returns HTML.
async fn render_page(name: &str) -> axum::response::Response {
    let page_path = format!("{}/{}", WEB_ROOT, name);
    let (layout_res, page_res) = tokio::join!(
        tokio::fs::read_to_string(LAYOUT_FILE),
        tokio::fs::read_to_string(&page_path),
    );
    let layout = match layout_res {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "layout missing").into_response(),
    };
    let content = match page_res {
        Ok(s) => s,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    Html(layout.replace("{{outlet}}", &content)).into_response()
}

async fn index() -> axum::response::Response {
    render_page("index.html").await
}

/// Layout file is a template, not a public page. Block direct access.
async fn forbid_layout() -> StatusCode {
    StatusCode::NOT_FOUND
}

fn create_app() -> Router{
    Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/.layout.html", get(forbid_layout))
        .route("/health", get(health_check))
        .fallback_service(ServeDir::new(WEB_ROOT))
}

#[tokio::main]
async fn main(){
    let app = create_app();
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("failed to bind tcp listener");

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
use tower::ServiceExt;

use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_app();

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.collect().await.unwrap();
        let json: Value = serde_json::from_slice(
            &body.to_bytes()
        ).unwrap();

        assert_eq!(json["status"], "ok");
        assert_eq!(json["message"], "Server is running");
    }

    #[tokio::test]
    async fn test_api_error_into_response(){
        let test_cases = vec![
            (ApiError::NotFound, StatusCode::NOT_FOUND),
            (
                ApiError::InvalidInput("Bad data".to_string()),
                StatusCode::BAD_REQUEST,
            ),
            (ApiError::InternalError, StatusCode::INTERNAL_SERVER_ERROR)
        ];

        for (error, expected_status) in test_cases{
            let response = error.into_response();
            assert_eq!(response.status(), expected_status)
        }
    }

    #[tokio::test]
    async fn test_index_renders_layout_with_content() {
        let app = create_app();

        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body.to_vec()).unwrap();

        // Layout chrome must be present
        assert!(html.contains("<!DOCTYPE html>"), "missing DOCTYPE from layout");
        assert!(html.contains("class=\"site-footer\""), "missing footer from layout");
        // Page fragment must be inlined where {{outlet}} was
        assert!(html.contains("class=\"content-container\""), "missing page fragment");
        // Placeholder must have been substituted
        assert!(!html.contains("{{outlet}}"), "{{outlet}} placeholder not substituted");
    }

    #[tokio::test]
    async fn test_layout_file_blocked() {
        let app = create_app();

        let request = Request::builder()
            .uri("/.layout.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
