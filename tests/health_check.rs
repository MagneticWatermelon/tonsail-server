use http::{Request, StatusCode};
use hyper::Body;
use tonsail_server::{configuration::get_configuration, Application};
use tower::ServiceExt;

#[tokio::test]
async fn returns_200_when_application_is_healthy() {
    let config = get_configuration().unwrap();
    let app = Application::build(config).await.unwrap();

    let response = app
        .router
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
