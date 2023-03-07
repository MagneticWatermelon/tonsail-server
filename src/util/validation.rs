use super::app_error::AppError;
use axum::{
    async_trait,
    extract::{
        rejection::{FormRejection, QueryRejection},
        FromRequest, Query,
    },
    Form,
};
use http::Request;
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, B, Rejection = FormRejection>,
    B: Send + 'static,
{
    type Rejection = AppError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Query<T>: FromRequest<S, B, Rejection = QueryRejection>,
    B: Send + 'static,
{
    type Rejection = AppError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedQuery(value))
    }
}
