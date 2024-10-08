//! Intermediate module for encapsulate all the redis related functionality
//!
//! Provides structs to represent redis connection and all functions that redis provides and
//! are used in the `router` crate. Abstractions for creating a new connection while also facilitating
//! redis connection pool and configuration based types.
//!
//!  # Examples
//! ```
//! use redis_interface::{types::RedisSettings, RedisConnectionPool};
//!
//! #[tokio::main]
//! async fn main() {
//!     let redis_conn = RedisConnectionPool::new(&RedisSettings::default()).await;
//!     // ... redis_conn ready to use
//! }
//! ```

pub mod commands;
pub mod errors;
pub mod types;

use std::sync::{atomic, Arc};

use common_utils::errors::CustomResult;
use error_stack::ResultExt;
pub use fred::interfaces::PubsubInterface;
use fred::{interfaces::ClientLike, prelude::EventInterface};

pub use self::types::*;

pub struct RedisConnectionPool {
    pub pool: Arc<fred::prelude::RedisPool>,
    pub key_prefix: String,
    pub config: Arc<RedisConfig>,
    pub subscriber: Arc<SubscriberClient>,
    pub publisher: Arc<RedisClient>,
    pub is_redis_available: Arc<atomic::AtomicBool>,
}

pub struct RedisClient {
    inner: fred::prelude::RedisClient,
}

impl std::ops::Deref for RedisClient {
    type Target = fred::prelude::RedisClient;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RedisClient {
    pub async fn new(
        config: fred::types::RedisConfig,
        reconnect_policy: fred::types::ReconnectPolicy,
        perf: fred::types::PerformanceConfig,
    ) -> CustomResult<Self, errors::RedisError> {
        let client =
            fred::prelude::RedisClient::new(config, Some(perf), None, Some(reconnect_policy));
        client.connect();
        client
            .wait_for_connect()
            .await
            .change_context(errors::RedisError::RedisConnectionError)?;
        Ok(Self { inner: client })
    }
}

pub struct SubscriberClient {
    inner: fred::clients::SubscriberClient,
    pub is_subscriber_handler_spawned: Arc<atomic::AtomicBool>,
}

impl SubscriberClient {
    pub async fn new(
        config: fred::types::RedisConfig,
        reconnect_policy: fred::types::ReconnectPolicy,
        perf: fred::types::PerformanceConfig,
    ) -> CustomResult<Self, errors::RedisError> {
        let client =
            fred::clients::SubscriberClient::new(config, Some(perf), None, Some(reconnect_policy));
        client.connect();
        client
            .wait_for_connect()
            .await
            .change_context(errors::RedisError::RedisConnectionError)?;
        Ok(Self {
            inner: client,
            is_subscriber_handler_spawned: Arc::new(atomic::AtomicBool::new(false)),
        })
    }
}

impl std::ops::Deref for SubscriberClient {
    type Target = fred::clients::SubscriberClient;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RedisConnectionPool {
    /// Create a new Redis connection
    pub async fn new(conf: &RedisSettings) -> CustomResult<Self, errors::RedisError> {
        let redis_connection_url = match conf.cluster_enabled {
            // Fred relies on this format for specifying cluster where the host port is ignored & only query parameters are used for node addresses
            // redis-cluster://username:password@host:port?node=bar.com:30002&node=baz.com:30003
            true => format!(
                "redis-cluster://{}:{}?{}",
                conf.host,
                conf.port,
                conf.cluster_urls
                    .iter()
                    .flat_map(|url| vec!["&", url])
                    .skip(1)
                    .collect::<String>()
            ),
            false => format!(
                "redis://{}:{}", //URI Schema
                conf.host, conf.port,
            ),
        };
        let mut config = fred::types::RedisConfig::from_url(&redis_connection_url)
            .change_context(errors::RedisError::RedisConnectionError)?;

        let perf = fred::types::PerformanceConfig {
            auto_pipeline: conf.auto_pipeline,
            default_command_timeout: std::time::Duration::from_secs(conf.default_command_timeout),
            max_feed_count: conf.max_feed_count,
            backpressure: fred::types::BackpressureConfig {
                disable_auto_backpressure: conf.disable_auto_backpressure,
                max_in_flight_commands: conf.max_in_flight_commands,
                policy: fred::types::BackpressurePolicy::Drain,
            },
        };

        let connection_config = fred::types::ConnectionConfig {
            unresponsive_timeout: std::time::Duration::from_secs(conf.unresponsive_timeout),
            ..fred::types::ConnectionConfig::default()
        };

        if !conf.use_legacy_version {
            config.version = fred::types::RespVersion::RESP3;
        }
        config.tracing = fred::types::TracingConfig::new(true);
        config.blocking = fred::types::Blocking::Error;
        let reconnect_policy = fred::types::ReconnectPolicy::new_constant(
            conf.reconnect_max_attempts,
            conf.reconnect_delay,
        );

        let subscriber =
            SubscriberClient::new(config.clone(), reconnect_policy.clone(), perf.clone()).await?;

        let publisher =
            RedisClient::new(config.clone(), reconnect_policy.clone(), perf.clone()).await?;

        let pool = fred::prelude::RedisPool::new(
            config,
            Some(perf),
            Some(connection_config),
            Some(reconnect_policy),
            conf.pool_size,
        )
        .change_context(errors::RedisError::RedisConnectionError)?;

        pool.connect();
        pool.wait_for_connect()
            .await
            .change_context(errors::RedisError::RedisConnectionError)?;

        let config = RedisConfig::from(conf);

        Ok(Self {
            pool: Arc::new(pool),
            config: Arc::new(config),
            is_redis_available: Arc::new(atomic::AtomicBool::new(true)),
            subscriber: Arc::new(subscriber),
            publisher: Arc::new(publisher),
            key_prefix: String::default(),
        })
    }
    pub fn clone(&self, key_prefix: &str) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            key_prefix: key_prefix.to_string(),
            config: Arc::clone(&self.config),
            subscriber: Arc::clone(&self.subscriber),
            publisher: Arc::clone(&self.publisher),
            is_redis_available: Arc::clone(&self.is_redis_available),
        }
    }
    pub async fn on_error(&self, tx: tokio::sync::oneshot::Sender<()>) {
        use futures::StreamExt;
        use tokio_stream::wrappers::BroadcastStream;

        let error_rxs: Vec<BroadcastStream<fred::error::RedisError>> = self
            .pool
            .clients()
            .iter()
            .map(|client| BroadcastStream::new(client.error_rx()))
            .collect();

        let mut error_rx = futures::stream::select_all(error_rxs);
        loop {
            if let Some(Ok(error)) = error_rx.next().await {
                tracing::error!(?error, "Redis protocol or connection error");
                if self.pool.state() == fred::types::ClientState::Disconnected {
                    if tx.send(()).is_err() {
                        tracing::error!("The redis shutdown signal sender failed to signal");
                    }
                    self.is_redis_available
                        .store(false, atomic::Ordering::SeqCst);
                    break;
                }
            }
        }
    }

    pub async fn on_unresponsive(&self) {
        let _ = self.pool.clients().iter().map(|client| {
            client.on_unresponsive(|server| {
                tracing::warn!(redis_server =?server.host, "Redis server is unresponsive");
                Ok(())
            })
        });
    }
}

pub struct RedisConfig {
    default_ttl: u32,
    default_stream_read_count: u64,
    default_hash_ttl: u32,
}

impl From<&RedisSettings> for RedisConfig {
    fn from(config: &RedisSettings) -> Self {
        Self {
            default_ttl: config.default_ttl,
            default_stream_read_count: config.stream_read_count,
            default_hash_ttl: config.default_hash_ttl,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_redis_error() {
        let x = errors::RedisError::ConsumerGroupClaimFailed.to_string();

        assert_eq!(x, "Failed to set Redis stream message owner".to_string())
    }
}
