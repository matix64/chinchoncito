mod chinchon;
mod comandos;
mod componentes;
mod config_servers;
mod crear_hilo;
mod errores;
mod estadisticas;
mod eventos;
mod handler;
mod lista_partidas;
mod mensajes;
mod opciones_comandos;

use crate::{
    config_servers::ConfigServers, estadisticas::Estadisticas, handler::Handler,
    lista_partidas::ListaPartidas,
};
use chinchon::inicializar_emojis_palos;
use serde::Deserialize;
use serenity::{client::Client, model::id::EmojiId, prelude::GatewayIntents};
use std::sync::Arc;
use tokio::{fs::File, io::AsyncReadExt, signal::ctrl_c, spawn};

#[derive(Deserialize)]
struct Config {
    token: String,
    redis: String,
    emojis: ConfigEmojis,
}

#[derive(Deserialize)]
struct ConfigEmojis {
    copa: EmojiId,
    espada: EmojiId,
    oro: EmojiId,
    basto: EmojiId,
}

#[tokio::main]
async fn main() {
    let config: Config = {
        let mut contenido = String::new();
        File::open("config.yml")
            .await
            .expect("Abrir config.yml")
            .read_to_string(&mut contenido)
            .await
            .expect("Leer config.yml");
        serde_yaml::from_str(&contenido).expect("Leer config.yml")
    };
    inicializar_emojis_palos(
        config.emojis.copa,
        config.emojis.espada,
        config.emojis.oro,
        config.emojis.basto,
    )
    .expect("Inicializar emojis de palos");
    let client_redis = redis::Client::open(config.redis).expect("Validar URL de redis");
    let con_redis = client_redis
        .get_multiplexed_tokio_connection()
        .await
        .expect("Conectar con redis");
    let partidas = Arc::new(ListaPartidas::cargar().await.expect("Cargar partidas"));
    let handler = {
        let configs = ConfigServers::new(con_redis.clone());
        let estadisticas = Estadisticas::new(con_redis.clone());
        Handler::new(partidas.clone(), configs, estadisticas)
    };
    let mut cliente = Client::builder(config.token, GatewayIntents::DIRECT_MESSAGES)
        .event_handler(handler.clone())
        .await
        .expect("Crear cliente");
    let shards = cliente.shard_manager.clone();
    spawn(async move {
        ctrl_c().await.expect("Recibir señal Ctrl+C");
        println!("Deteniendo...");
        shards.lock().await.shutdown_all().await;
    });
    cliente.start().await.expect("Iniciar cliente");
    println!("Esperando a que finalicen los comandos...");
    handler.detener().await;
    println!("Guardando partidas...");
    partidas.guardar().await.expect("Guardar partidas");
    println!("Listo! bye <3");
}
