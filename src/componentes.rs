use crate::{
    chinchon::{Carta, Partida, PilaCartas},
    crear_hilo::crear_hilo_partida,
    errores::ErrorGenerico,
    estadisticas::Estadisticas,
    eventos::{fin_partida, perdio},
    lista_partidas::ListaPartidas,
    mensajes::{mensaje_cartas, mensaje_cortar, mensaje_invitacion, mensaje_jugar},
};
use anyhow::{anyhow, Result};
use serenity::{
    client::Context,
    model::{
        id::{MessageId, UserId},
        interactions::{message_component::MessageComponentInteraction, InteractionResponseType},
    },
};
use tokio::sync::MutexGuard;

pub async fn inter_componente(
    ctx: &Context,
    inter: &mut MessageComponentInteraction,
    partidas: &ListaPartidas,
    estadisticas: &mut Estadisticas,
) -> Result<()> {
    match inter.data.custom_id.as_str() {
        "aceptar inv" => {
            let creador_invi = inter
                .message
                .interaction
                .as_ref()
                .ok_or(())
                .error_generico()?
                .user
                .id;
            let partida = partidas
                .aceptar_invitacion(
                    inter.channel_id,
                    creador_invi,
                    inter.message.id,
                    inter.user.id,
                )
                .await?;
            let (contenido, acciones) = mensaje_invitacion(
                &ctx.http,
                creador_invi,
                &partida.jugadores(),
                partida.max_jugadores as u64,
                partida.privada(),
            )
            .await;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.kind(InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|msg| {
                            msg.content(contenido)
                                .components(|comps| comps.set_action_rows(acciones))
                        })
                })
                .await
                .unwrap();
            if partida.llena() {
                let http = ctx.http.clone();
                let canal = inter.channel_id;
                partidas
                    .empezar_partida(
                        canal,
                        creador_invi,
                        |mensaje: MessageId, jugadores: Vec<UserId>, comienza: UserId| async move {
                            crear_hilo_partida(&http, canal, mensaje, &jugadores, comienza).await
                        },
                    )
                    .await
                    .error_generico()?;
            }
        }
        comando if comando.starts_with("jugar") => {
            let para: UserId = comando.replace("jugar ", "").parse().error_generico()?;
            if para != inter.user.id {
                return Err(anyhow!("Este boton no es para ti :( lo siento chiquito"));
            }
            let partida = partidas
                .get_partida(inter.channel_id)
                .await
                .ok_or_else(|| anyhow!("No encuentro esta partida :c"))?;
            let mut partida = partida.lock().await;
            let jugador = partida
                .jugador(inter.user.id)
                .ok_or_else(|| anyhow!("No estas en esta partida corazon :c"))?;
            if !jugador.es_turno() {
                return Err(anyhow!("Que tontito sempaii >_< aun no te toca"));
            }
            let (mensaje, acciones) = mensaje_jugar(&jugador, None, None);
            drop(partida);
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.ephemeral(true)
                            .content(mensaje)
                            .components(|comp| comp.set_action_rows(acciones))
                    })
                })
                .await
                .error_generico()?;
            inter
                .message
                .edit(&ctx.http, |msg| {
                    msg.components(|comps| comps.set_action_rows(vec![]))
                })
                .await
                .error_generico()?;
        }
        "levantar mazo" | "levantar descarte" => {
            let pila = match inter.data.custom_id.as_str() {
                "levantar mazo" => PilaCartas::Mazo,
                _ => PilaCartas::Descartes,
            };
            let partida = partidas
                .get_partida(inter.channel_id)
                .await
                .ok_or_else(|| anyhow!("No encuentro esta partida :c"))?;
            let mut partida = partida.lock().await;
            let mut jugador = partida
                .jugador(inter.user.id)
                .ok_or_else(|| anyhow!("No estas en esta partida corazon :c"))?;
            let levantada = jugador
                .levantar(pila)
                .map_err(|_| anyhow!("No puedes levantar ahora :("))?;
            let (mensaje, acciones) = mensaje_jugar(&jugador, Some(levantada), None);
            drop(partida);
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.kind(InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|msg| {
                            msg.content(mensaje)
                                .components(|comps| comps.set_action_rows(acciones))
                        })
                })
                .await
                .error_generico()?;
        }
        "selec carta" => {
            let carta_selec: Carta = inter.data.values[0].parse().error_generico()?;
            let partida = partidas
                .get_partida(inter.channel_id)
                .await
                .ok_or_else(|| anyhow!("No encuentro esta partida :c"))?;
            let mut partida = partida.lock().await;
            let jugador = partida
                .jugador(inter.user.id)
                .ok_or_else(|| anyhow!("No estas en esta partida corazon :c"))?;
            if jugador.puede_cortar(carta_selec) {
                let (_, acciones) = mensaje_jugar(&jugador, None, Some(carta_selec));
                drop(partida);
                inter
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|msg| {
                                msg.components(|comps| comps.set_action_rows(acciones))
                            })
                    })
                    .await
                    .error_generico()?;
            } else {
                bajar(ctx, inter, partida, carta_selec).await?;
            }
        }
        comando if comando.starts_with("bajar") => {
            let carta: Carta = comando.replace("bajar ", "").parse().error_generico()?;
            let partida = partidas
                .get_partida(inter.channel_id)
                .await
                .ok_or_else(|| anyhow!("No encuentro esta partida :c"))?;
            bajar(ctx, inter, partida.lock().await, carta).await?;
        }
        comando if comando.starts_with("cortar") => {
            let guild_id = inter.guild_id.unwrap();
            let carta: Carta = comando.replace("cortar ", "").parse().error_generico()?;
            let partida = partidas
                .get_partida(inter.channel_id)
                .await
                .ok_or_else(|| anyhow!("No encuentro esta partida :c"))?;
            let mut partida = partida.lock().await;
            let mut jugador = partida
                .jugador(inter.user.id)
                .ok_or_else(|| anyhow!("No estas en esta partida corazon :c"))?;
            let resultados = jugador.cortar(Some(carta)).error_generico()?;
            let turno = partida.get_turno();
            let ganador = partida.ganador();
            drop(partida);
            let (mensaje, acciones) = mensaje_cortar(
                &ctx.http,
                &resultados,
                &inter.user,
                (ganador.is_none()).then(|| turno),
            )
            .await;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.content(mensaje)
                            .components(|comps| comps.set_action_rows(acciones))
                    })
                })
                .await
                .error_generico()?;
            for res in &resultados {
                if res.perdio {
                    perdio(estadisticas, guild_id, res.jugador).await;
                }
            }
            if let Some(ganador) = ganador {
                partidas.terminar_partida(inter.channel_id).await?;
                fin_partida(
                    &ctx.http,
                    estadisticas,
                    inter.guild_id.unwrap(),
                    inter.channel_id,
                    ganador,
                )
                .await;
            }
        }
        _ => return Err(anyhow!("chica q dices")),
    }
    Ok(())
}

async fn bajar(
    ctx: &Context,
    inter: &MessageComponentInteraction,
    mut partida: MutexGuard<'_, Partida>,
    carta: Carta,
) -> Result<()> {
    let mut jugador = partida
        .jugador(inter.user.id)
        .ok_or_else(|| anyhow!("No estas en esta partida corazon :c"))?;
    jugador
        .tirar(carta)
        .map_err(|_| anyhow!("No puedes tirar ahora unu"))?;
    let cartas = jugador.get_cartas();
    let turno = partida.get_turno();
    let pila_levante = partida.get_pila_ultimo_levante();
    drop(partida);
    inter
        .create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|msg| {
                    msg.content(mensaje_cartas(&cartas))
                        .components(|comps| comps.set_action_rows(vec![]))
                })
        })
        .await
        .error_generico()?;
    inter
        .create_followup_message(&ctx.http, |msg| {
            msg.content(format!(
                "{}-sama {}tiro un {}\n<@{}> te toca uwu",
                inter.user.name,
                match pila_levante {
                    Some(PilaCartas::Mazo) => "levanto del mazo y ",
                    Some(PilaCartas::Descartes) => "levanto el descarte y ",
                    None => "",
                },
                carta,
                turno
            ))
            .components(|comps| {
                comps.create_action_row(|row| {
                    row.create_button(|btn| {
                        btn.custom_id(format!("jugar {}", turno)).label("Jugar")
                    })
                })
            })
        })
        .await
        .error_generico()?;
    Ok(())
}
