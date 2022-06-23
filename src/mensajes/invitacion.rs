use serenity::{builder::CreateActionRow, http::CacheHttp, model::id::UserId};
use std::fmt::Write;

pub async fn mensaje_invitacion(
    http: &impl CacheHttp,
    creador: UserId,
    jugadores: &[UserId],
    max_jugadores: u64,
    privada: bool,
) -> (String, Vec<CreateActionRow>) {
    let mut cont = format!(
        "<@{}> esta buscando alguien pa jugar un chinchocito 😳",
        creador
    );
    if privada {
        cont += "\nLa mesa es privada, usa **/invitar** para agregar gente";
    }
    let mut nombres = Vec::with_capacity(jugadores.len());
    for j in jugadores {
        nombres.push(
            j.to_user(http)
                .await
                .map(|u| u.name)
                .unwrap_or_else(|_| "?".to_owned()),
        );
    }
    let _ = write!(
        cont,
        "\n**Jugadores ({}/{}):**\n{}",
        jugadores.len(),
        max_jugadores,
        nombres.join(", ")
    );
    let mut acciones = vec![];
    if (jugadores.len() as u64) < max_jugadores {
        let mut row = CreateActionRow::default();
        row.create_button(|btn| btn.label("Unirse").custom_id("aceptar inv"));
        acciones.push(row);
    }
    (cont, acciones)
}
