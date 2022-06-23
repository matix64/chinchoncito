mod info;
mod jugador;

use crate::{chinchon::Partida, estadisticas::Estadisticas};
use anyhow::{anyhow, Result};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};
use std::{fmt::Write, sync::Arc};
use tokio::sync::Mutex;

use self::jugador::comando_jugador;

pub async fn comando_partida(
    ctx: &Context,
    inter: &ApplicationCommandInteraction,
    partida: Arc<Mutex<Partida>>,
    estadisticas: &mut Estadisticas,
) -> Result<()> {
    match inter.data.name.as_str() {
        "jugar" | "cartas" | "salir" | "kick" => {
            let mut partida = partida.lock().await;
            let mut jugador = partida
                .jugador(inter.user.id)
                .ok_or_else(|| anyhow!("No estas en esta partida corazon :c"))?;
            comando_jugador(ctx, inter, &mut jugador, estadisticas).await?;
        }
        "puntos" => {
            let puntos: Vec<_> = {
                let partida = partida.lock().await;
                partida.get_puntos().into_iter().collect()
            };
            let mut tabla = String::new();
            for (user_id, puntos) in puntos {
                let _ = writeln!(
                    tabla,
                    "**{}**: {}",
                    user_id
                        .to_user(&ctx.http)
                        .await
                        .map(|u| u.name)
                        .unwrap_or_else(|_| "?".to_owned()),
                    puntos
                );
            }
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.ephemeral(true).content(tabla.to_string())
                    })
                })
                .await
                .unwrap();
        }
        _ => return Err(anyhow!("chica q dices")),
    }
    Ok(())
}
