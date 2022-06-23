use crate::errores::ErrorGenerico;
use anyhow::Result;
use serenity::{
    http::Http,
    model::id::{ChannelId, MessageId, UserId},
};

pub async fn crear_hilo_partida(
    http: &Http,
    canal: ChannelId,
    mensaje: MessageId,
    jugadores: &[UserId],
    comienza: UserId,
) -> Result<ChannelId> {
    let mut nombres = Vec::with_capacity(jugadores.len());
    for j in jugadores {
        nombres.push(j.to_user(&http).await.error_generico()?.name);
    }
    let nombre_canal: String = format!("Chinchon {}", nombres.join(" vs "))
        .chars()
        .take(100)
        .collect();
    let canal = canal
        .create_public_thread(&http, mensaje, |t| {
            t.name(nombre_canal).auto_archive_duration(60)
        })
        .await
        .map(|canal| canal.id)
        .error_generico()?;
    canal
        .send_message(http, |msg| {
            msg.content(format!(
                "Empieza la partida {menciones} uwu\n\
                 Es tu turno <@{comienza}>, usa **/jugar** para empezar",
                menciones = jugadores
                    .iter()
                    .map(|id| format!("<@{id}>"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        })
        .await
        .error_generico()?;
    Ok(canal)
}
