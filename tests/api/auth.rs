use http::{Request, StatusCode};
use hyper::Body;
use prisma_client_rust::serde_json;
use serde_urlencoded::to_string;
use tonsail_server::{
    configuration::get_configuration,
    domain::auth::{AuthRegisterForm, TonsailUser},
    Application,
};
use tower::ServiceExt;

use crate::util::seed_database;

#[tokio::test]
async fn returns_200_with_user_when_registration_is_succesfull() {
    let config = get_configuration().unwrap();
    let app = Application::build(config).await.unwrap();

    let body = Body::from(
        to_string(AuthRegisterForm {
            name: "Marie Curie".to_string(),
            email: "marie@curie.com".to_string(),
            password: "M@rieCurie69".to_string(),
        })
        .unwrap(),
    );
    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .uri("/register")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice::<TonsailUser>(&body).expect("Could not parse body into User");
}

#[tokio::test]
async fn returns_400_when_user_already_exists() {
    let config = get_configuration().unwrap();
    let app = Application::build(config).await.unwrap();

    seed_database().await;

    let body = Body::from(
        to_string(AuthRegisterForm {
            name: "Graham Bell".to_string(),
            email: "graham@bell.com".to_string(),
            password: "Gr@h@mBell69".to_string(),
        })
        .unwrap(),
    );
    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("POST")
                .header(
                    http::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .uri("/register")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
