use anyhow::Result;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serenity::model::id::{GuildId, UserId};

#[derive(Clone)]
pub struct Estadisticas {
    redis: MultiplexedConnection,
}

#[derive(Clone, Debug)]
pub struct EstadisticasJugador {
    pub victorias: u64,
    pub derrotas: u64,
}

impl Estadisticas {
    pub fn new(redis: MultiplexedConnection) -> Self {
        Self { redis }
    }

    pub async fn agregar_victoria(&mut self, server: GuildId, jugador: UserId) -> Result<()> {
        let vict_totales = format!("victorias:total:{}", jugador);
        let vict_server = format!("victorias:{}:{}", server, jugador);
        for clave in [vict_totales, vict_server] {
            self.redis.incr(clave, 1).await?;
        }
        Ok(())
    }

    pub async fn agregar_derrota(&mut self, server: GuildId, jugador: UserId) -> Result<()> {
        let derr_totales = format!("derrotas:total:{}", jugador);
        let derr_server = format!("derrotas:{}:{}", server, jugador);
        for clave in [derr_totales, derr_server] {
            self.redis.incr(clave, 1).await?;
        }
        Ok(())
    }

    pub async fn get(
        &self,
        jugador: UserId,
        server: Option<GuildId>,
    ) -> Result<EstadisticasJugador> {
        let mut redis = self.redis.clone();
        let server = server
            .map(|s| s.to_string())
            .unwrap_or_else(|| "total".to_owned());
        Ok(EstadisticasJugador {
            victorias: redis
                .get::<_, Option<u64>>(format!("victorias:{server}:{jugador}"))
                .await?
                .unwrap_or_default(),
            derrotas: redis
                .get::<_, Option<u64>>(format!("derrotas:{server}:{jugador}"))
                .await?
                .unwrap_or_default(),
        })
    }
}
