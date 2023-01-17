use http::{Request, StatusCode};
use hyper::Body;
use tonsail_server::{
    prisma::PrismaClient,
    routes::{create_router, AppState},
};
use tower::ServiceExt;

#[tokio::test]
async fn returns_200_when_application_is_healthy() {
    let (client, _mock) = PrismaClient::_mock();
    let state = AppState::new(client);
    let app = create_router(state);

    let response = app
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
