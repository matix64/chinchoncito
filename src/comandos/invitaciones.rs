use crate::{
    config_servers::ConfigServers,
    crear_hilo::crear_hilo_partida,
    lista_partidas::{ErrorEmpezarPartida, ListaPartidas, RespuestaEmpezarPartida},
    mensajes::mensaje_invitacion,
    opciones_comandos::{get_opcion, get_opcion_o_default},
};
use anyhow::{anyhow, Result};
use serenity::{
    client::Context,
    model::{
        id::{MessageId, UserId},
        interactions::application_command::ApplicationCommandInteraction,
        user::User,
    },
};

pub async fn comando_invitacion(
    ctx: &Context,
    inter: &ApplicationCommandInteraction,
    partidas: &ListaPartidas,
    config_servers: &ConfigServers,
) -> Result<()> {
    match inter.data.name.as_str() {
        "chinchon" => {
            if !config_servers
                .puede_crear_partidas(inter.guild_id.unwrap(), inter.channel_id)
                .await
                .unwrap()
            {
                return Err(anyhow!(
                    "No se pueden crear partidas en este canal :(\n\
                    Un admin puede elegir el canal para crear partidas usando **/canal #nombre**"
                ));
            }
            let privada = get_opcion_o_default("privada", inter, false)?;
            let max_jugadores = get_opcion_o_default("jugadores", inter, 2)?.clamp(2, 4);
            let (contenido, acciones) = mensaje_invitacion(
                &ctx.http,
                inter.user.id,
                &[inter.user.id],
                max_jugadores as u64,
                privada,
            )
            .await;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| {
                        msg.content(contenido)
                            .components(|comps| comps.set_action_rows(acciones))
                    })
                })
                .await
                .unwrap();
            let mensaje = inter.get_interaction_response(&ctx.http).await.unwrap();
            let inv_vieja = partidas
                .crear_invitacion(
                    inter.channel_id,
                    inter.user.id,
                    mensaje.id,
                    if privada { Some(vec![]) } else { None },
                    max_jugadores as usize,
                )
                .await;
            if let Some((canal, mensaje)) = inv_vieja {
                ctx.http.delete_message(canal.0, mensaje.0).await.unwrap();
            }
        }
        "invitar" => {
            let invitado: User = get_opcion("a", inter)?;
            partidas
                .agregar_invitado(inter.channel_id, inter.user.id, invitado.id)
                .await?;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| msg.ephemeral(true).content("Listo ^^"))
                })
                .await
                .unwrap();
        }
        "empezar" => {
            let http = ctx.http.clone();
            let canal = inter.channel_id;
            let RespuestaEmpezarPartida { mensaje_invi, .. } = partidas
                .empezar_partida(
                    inter.channel_id,
                    inter.user.id,
                    |mensaje: MessageId, jugadores: Vec<UserId>, comienza: UserId| async move {
                        crear_hilo_partida(&http, canal, mensaje, &jugadores, comienza).await
                    },
                )
                .await
                .map_err(|e| match e {
                    ErrorEmpezarPartida::InvitacionNoExiste => {
                        anyhow!("No creaste ninguna invitacion, usa **/chinchon** para crear una")
                    }
                    ErrorEmpezarPartida::PocosJugadores => {
                        anyhow!("Debes esperar a que se una alguien mas")
                    }
                    ErrorEmpezarPartida::ErrorCreandoCanal => {
                        anyhow!("Algo salio mal bb :( no se pudo crear el hilo")
                    }
                })?;
            inter
                .create_interaction_response(&ctx.http, |resp| {
                    resp.interaction_response_data(|msg| msg.ephemeral(true).content("Listo ^^"))
                })
                .await
                .unwrap();
            inter
                .channel_id
                .edit_message(&ctx.http, mensaje_invi, |msg| {
                    msg.components(|comps| comps.set_action_rows(vec![]))
                })
                .await
                .unwrap();
        }
        _ => return Err(anyhow!("chica q dices")),
    }
    Ok(())
}
