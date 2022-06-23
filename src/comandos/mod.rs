mod invitaciones;
mod partida;

use self::{invitaciones::comando_invitacion, partida::comando_partida};
use crate::{
    config_servers::ConfigServers,
    errores::ErrorGenerico,
    estadisticas::Estadisticas,
    eventos::fin_partida,
    lista_partidas::ListaPartidas,
    mensajes::mensaje_estadisticas,
    opciones_comandos::{get_opcion, get_opcion_o_default},
};
use anyhow::{anyhow, Result};
use serenity::{
    client::Context,
    model::{
        channel::PartialChannel,
        interactions::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        user::User,
    },
};
use std::time::Duration;
use tokio::time::sleep;

pub async fn procesar_comando(
    ctx: &Context,
    inter: &ApplicationCommandInteraction,
    partidas: &ListaPartidas,
    config_servers: &mut ConfigServers,
    estadisticas: &mut Estadisticas,
) -> Result<()> {
    match inter.data.name.as_str() {
        "test" => {
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                        .interaction_response_data(|msg| msg.ephemeral(true))
                })
                .await
                .unwrap();
            sleep(Duration::from_secs(10)).await;
            inter
                .edit_original_interaction_response(&ctx.http, |msg| msg.content("Listo :D"))
                .await
                .unwrap();
        }
        "canal" => {
            let canal: PartialChannel = get_opcion("canal", inter)?;
            config_servers
                .set_canal_partidas(inter.guild_id.unwrap(), canal.id)
                .await
                .error_generico()?;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| msg.content("Listo ^^").ephemeral(true))
                })
                .await
                .unwrap();
        }
        "stats" => {
            let jugador: User = get_opcion_o_default("jugador", inter, inter.user.clone())?;
            let embed = mensaje_estadisticas(&jugador, inter.guild_id, estadisticas).await?;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| msg.set_embed(embed))
                })
                .await
                .unwrap();
        }
        "chinchon" | "invitar" | "empezar" => {
            comando_invitacion(ctx, inter, partidas, config_servers).await?
        }
        "jugar" | "puntos" | "cartas" | "salir" | "kick" => {
            let partida = partidas
                .get_partida(inter.channel_id)
                .await
                .ok_or_else(|| anyhow!("No hay ninguna partida en este canal :("))?;
            comando_partida(ctx, inter, partida.clone(), estadisticas).await?;
            if let Some(ganador) = partida.lock().await.ganador() {
                partidas.terminar_partida(inter.channel_id).await.unwrap();
                fin_partida(
                    &ctx.http,
                    estadisticas,
                    inter.guild_id.unwrap(),
                    inter.channel_id,
                    ganador,
                )
                .await;
            };
        }
        _ => return Err(anyhow!("nani?")),
    }
    Ok(())
}
