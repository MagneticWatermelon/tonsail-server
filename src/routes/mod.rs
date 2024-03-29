use self::auth::{check_me, login, logout, register_new_user};
use self::layers::{add_auth_layer, add_cors_layer, add_trace_layer};
use self::metrics::{get_metrics, get_metrics_catalog};
use self::organizations::{get_organizations, update_organization};
use self::project::{create_project, get_project, update_project};
use self::test_run::{create_test_run, get_test_run};
use self::tests::{create_test, get_test};
use self::user::{get_user, update_password, update_user};
use crate::domain::auth::TonsailUser;
use crate::AppState;
use axum::routing::{get, post, put};
use axum::Router;
use axum_login::RequireAuthorizationLayer;
use health_check::health_check;
use organizations::get_organization;

pub mod auth;
pub mod health_check;
pub mod layers;
pub mod metrics;
pub mod organizations;
pub mod project;
pub mod test_run;
pub mod tests;
pub mod user;

pub fn create_router(state: AppState) -> Router {
    let mut app = Router::new()
        .route("/me", get(check_me))
        .route("/logout", post(logout))
        .route("/metrics", get(get_metrics))
        .route("/metrics/catalog", get(get_metrics_catalog))
        .route("/users/:user_id", get(get_user).put(update_user))
        .route("/users/:user_id/password", put(update_password))
        .route("/runs/:run_id", get(get_test_run).post(create_test_run))
        .route("/tests", post(create_test))
        .route("/tests/:test_id", get(get_test))
        .route("/projects", post(create_project))
        .route(
            "/projects/:project_id",
            get(get_project).put(update_project),
        )
        .route("/organizations", get(get_organizations))
        .route(
            "/organizations/:organization_id",
            get(get_organization).put(update_organization),
        )
        .route_layer(RequireAuthorizationLayer::<TonsailUser>::login())
        .route("/login", post(login))
        .route("/register", post(register_new_user))
        .route("/health_check", get(health_check));
    app = add_cors_layer(app);
    app = add_auth_layer(app, state.clone());
    app = add_trace_layer(app);
    app.with_state(state)
}
