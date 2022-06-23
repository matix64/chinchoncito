use crate::{
    chinchon::Jugador,
    estadisticas::Estadisticas,
    eventos::abandono,
    mensajes::{mensaje_cartas, mensaje_jugar},
    opciones_comandos::get_opcion,
};
use anyhow::{anyhow, Result};
use serenity::{
    client::Context,
    model::{interactions::application_command::ApplicationCommandInteraction, user::User},
};

pub async fn comando_jugador(
    ctx: &Context,
    inter: &ApplicationCommandInteraction,
    jugador: &mut Jugador<'_>,
    estadisticas: &mut Estadisticas,
) -> Result<()> {
    match inter.data.name.as_str() {
        "jugar" => {
            if !jugador.es_turno() {
                return Err(anyhow!("Que tontito sempaii >_< aun no te toca"));
            }
            let (mensaje, acciones) = mensaje_jugar(jugador, None, None);
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.ephemeral(true)
                            .content(mensaje)
                            .components(|comp| comp.set_action_rows(acciones))
                    })
                })
                .await
                .unwrap();
        }
        "cartas" => {
            let cartas = jugador.get_cartas();
            let mensaje = mensaje_cartas(&cartas);
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| msg.ephemeral(true).content(mensaje))
                })
                .await
                .unwrap();
        }
        "salir" => {
            let turno_antes = jugador.partida.get_turno();
            jugador.abandonar();
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.ephemeral(true).content("Listo :( nos vemos guapurita")
                    })
                })
                .await
                .unwrap();
            let cambio_turno = if jugador.partida.ganador().is_some() {
                None
            } else {
                Some(jugador.partida.get_turno()).filter(|t| *t != turno_antes)
            };
            abandono(
                &ctx.http,
                estadisticas,
                inter.guild_id.unwrap(),
                inter.channel_id,
                inter.user.id,
                cambio_turno,
            )
            .await;
        }
        "kick" => {
            let victima: User = get_opcion("a", inter)?;
            let turno_antes = jugador.partida.get_turno();
            let expulsado = jugador.votar_expulsar_a(victima.id)?;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.ephemeral(true).content(format!(
                            "Listo{}",
                            if expulsado {
                                " >:)"
                            } else {
                                ", pero faltan mas votos"
                            }
                        ))
                    })
                })
                .await
                .unwrap();
            if expulsado {
                let cambio_turno = if jugador.partida.ganador().is_some() {
                    None
                } else {
                    Some(jugador.partida.get_turno()).filter(|t| *t != turno_antes)
                };
                abandono(
                    &ctx.http,
                    estadisticas,
                    inter.guild_id.unwrap(),
                    inter.channel_id,
                    victima.id,
                    cambio_turno,
                )
                .await;
            }
        }
        _ => return Err(anyhow!("chica q dices")),
    }
    Ok(())
}
