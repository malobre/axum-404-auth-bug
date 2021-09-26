use axum::{
    handler::{get, Handler},
    http::StatusCode,
    routing::BoxRoute,
    Router,
};
use tower_http::auth::RequireAuthorizationLayer;

pub fn app() -> Router<BoxRoute> {
    let public = Router::new().route("/public", get(|| async { StatusCode::OK }));

    let authorized = Router::new()
        .route("/authorized", get(|| async { StatusCode::OK }))
        .layer(RequireAuthorizationLayer::bearer("token"));

    let not_found = (|| async { StatusCode::NOT_FOUND }).into_service();

    public.or(authorized).or(not_found).boxed()

    // I also tested these:
    //public.or(not_found).or(authorized).boxed()
    //authorized.or(public).or(not_found).boxed()
    //authorized.or(not_found).or(public).boxed()
    //Router::new().or(not_found).or(authorized).or(public).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http;
    use tower::ServiceExt; // for `app.oneshot()`

    #[tokio::test]
    async fn test_public() {
        let app = app();

        let response = app
            .oneshot(
                http::Request::builder()
                    .uri("/public")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_unauthorized() {
        let app = app();

        let response = app
            .oneshot(
                http::Request::builder()
                    .uri("/authorized")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_authorized() {
        let app = app();

        let response = app
            .oneshot(
                http::Request::builder()
                    .uri("/authorized")
                    .header(
                        http::header::AUTHORIZATION,
                        http::HeaderValue::from_static("Bearer token"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = app();

        let response = app
            .oneshot(
                http::Request::builder()
                    .uri("/non_existent_route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
