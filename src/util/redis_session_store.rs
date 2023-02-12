use axum_login::axum_sessions::async_session::{
    async_trait, serde_json, Result, Session, SessionStore,
};
use fred::types::Scanner;
use fred::{
    pool::RedisPool,
    prelude::*,
    types::{RedisKey, ScanType},
};
use futures::stream::StreamExt;

#[derive(Clone)]
pub struct RedisSessionStore {
    pool: RedisPool,
    prefix: Option<String>,
}

impl std::fmt::Debug for RedisSessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.prefix)
    }
}

impl RedisSessionStore {
    pub fn from_pool(pool: RedisPool, prefix: Option<String>) -> Self {
        Self { pool, prefix }
    }

    pub async fn count(&self) -> Result<usize> {
        match self.prefix {
            None => Ok(self.pool.dbsize().await?),
            Some(_) => Ok(self.ids().await?.map_or(0, |v| v.len())),
        }
    }

    async fn ids(&self) -> Result<Option<Vec<RedisKey>>> {
        let mut result = Vec::new();
        let mut scanner = self
            .pool
            .scan(self.prefix_key("*"), None, Some(ScanType::String));

        while let Some(res) = scanner.next().await {
            if let Some(keys) = res?.take_results() {
                result.extend_from_slice(&keys);
            }
        }

        Ok((!result.is_empty()).then_some(result))
    }

    fn prefix_key(&self, key: &str) -> String {
        match &self.prefix {
            None => key.to_string(),
            Some(prefix) => format!("{prefix}{key}"),
        }
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        Ok(self
            .pool
            .get::<Option<String>, String>(self.prefix_key(&id))
            .await?
            .map(|v| serde_json::from_str(&v))
            .transpose()?)
    }

    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let id = self.prefix_key(session.id());
        let string = serde_json::to_string(&session)?;
        let expiration = session
            .expires_in()
            .map(|d| Expiration::EX(d.as_secs() as i64));

        self.pool.set(id, string, expiration, None, false).await?;

        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> Result {
        Ok(self.pool.del(self.prefix_key(session.id())).await?)
    }

    async fn clear_store(&self) -> Result {
        match self.prefix {
            None => Ok(self.pool.flushall(false).await?),
            Some(_) => match self.ids().await? {
                None => Ok(()),
                Some(ids) => Ok(self.pool.del(ids).await?),
            },
        }
    }
}
