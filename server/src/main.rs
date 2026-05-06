use axum::{Json, Router, extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::{Value, json};
use http_body_util::BodyExt;

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

async fn list_users() -> Result<Json<Value>, ApiError> {
    Err(ApiError::InternalError)
}

async fn get_user(Path(id): Path<u32>) -> Result<Json<Value>, ApiError> {
    if id > 100 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(json!({
        "id": id,
        "name": "User"
    })))
}

fn create_app() -> Router{
    Router::new()
        .route("/health", get(health_check))
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
}

#[tokio::main]
async fn main(){
    let app = create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
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
}