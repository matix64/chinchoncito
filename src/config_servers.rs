use anyhow::Result;
use redis::{aio::MultiplexedConnection, AsyncCommands, FromRedisValue};
use serenity::model::id::{ChannelId, GuildId};

#[derive(Clone)]
pub struct ConfigServers {
    redis: MultiplexedConnection,
}

impl ConfigServers {
    pub fn new(redis: MultiplexedConnection) -> Self {
        Self { redis }
    }

    pub async fn set_canal_partidas(&mut self, guild: GuildId, canal: ChannelId) -> Result<()> {
        self.redis.set(key_canal_partidas(guild), canal.0).await?;
        Ok(())
    }

    pub async fn puede_crear_partidas(&self, guild: GuildId, canal: ChannelId) -> Result<bool> {
        let val = self.redis.clone().get(key_canal_partidas(guild)).await?;
        if let Some(canal_config) = <Option<u64>>::from_redis_value(&val)? {
            Ok(canal_config == canal.0)
        } else {
            Ok(false)
        }
    }
}

fn key_canal_partidas(guild: GuildId) -> String {
    format!("canal_partidas:{}", guild)
}
