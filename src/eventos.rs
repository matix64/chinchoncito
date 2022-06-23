use crate::{estadisticas::Estadisticas, mensajes::mensaje_fin_partida};
use serenity::{
    builder::CreateActionRow,
    http::Http,
    model::id::{ChannelId, GuildId, UserId},
};

pub async fn fin_partida(
    http: &Http,
    estadisticas: &mut Estadisticas,
    server: GuildId,
    canal: ChannelId,
    ganador: UserId,
) {
    estadisticas
        .agregar_victoria(server, ganador)
        .await
        .unwrap();
    let mensaje = mensaje_fin_partida(http, server, ganador).await;
    canal
        .send_message(http, |msg| msg.set_embed(mensaje))
        .await
        .unwrap();
}

pub async fn abandono(
    http: &Http,
    estadisticas: &mut Estadisticas,
    server: GuildId,
    canal: ChannelId,
    jugador: UserId,
    cambio_turno: Option<UserId>,
) {
    let nombre_jugador = jugador.to_user(http).await.unwrap().name;
    let mut cont = format!("**{}** abandono la partida u.u", nombre_jugador);
    let mut accs = vec![];
    if let Some(turno) = cambio_turno {
        cont += &format!("\nAhora es el turno de <@{}>", turno);
        let mut row = CreateActionRow::default();
        row.create_button(|btn| btn.custom_id(format!("jugar {}", turno)).label("Jugar"));
        accs.push(row);
    }
    canal
        .send_message(http, |msg| {
            msg.content(cont)
                .components(|comps| comps.set_action_rows(accs))
        })
        .await
        .unwrap();
    perdio(estadisticas, server, jugador).await;
}

pub async fn perdio(estadisticas: &mut Estadisticas, server: GuildId, jugador: UserId) {
    estadisticas.agregar_derrota(server, jugador).await.unwrap();
}
