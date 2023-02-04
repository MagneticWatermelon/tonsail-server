use std::error::Error;

use prisma_client_rust::NewClientError;
use tracing::instrument;

use crate::prisma::PrismaClient;

#[derive(Debug)]
enum RetryResult<T, E>
where
    E: Error,
{
    Ok(T),
    Retry,
    Error(E),
}

pub async fn try_build_prisma(max_tries: u8) -> Result<PrismaClient, String> {
    for current_try in 0..max_tries {
        match connect_prisma(current_try, max_tries).await {
            RetryResult::Ok(c) => return Ok(c),
            RetryResult::Retry => continue,
            RetryResult::Error(e) => return Err(e.to_string()),
        }
    }
    Err("Something went wrong while connecting to database".to_string())
}

#[instrument(name = "Trying connecting")]
async fn connect_prisma(
    current_try: u8,
    max_tries: u8,
) -> RetryResult<PrismaClient, NewClientError> {
    match PrismaClient::_builder().build().await {
        Ok(client) => RetryResult::Ok(client),
        Err(_) if current_try < max_tries => RetryResult::Retry,
        Err(e) => RetryResult::Error(e),
    }
}
