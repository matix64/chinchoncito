use super::lista_cartas;
use crate::chinchon::ResultadoFinalRonda;
use serenity::{
    builder::CreateActionRow,
    http::CacheHttp,
    model::{id::UserId, user::User},
};
use std::{fmt::Write, iter};

pub async fn mensaje_cortar(
    http: impl CacheHttp,
    resultados: &[ResultadoFinalRonda],
    corto: &User,
    prox_turno: Option<UserId>,
) -> (String, Vec<CreateActionRow>) {
    let chinchon = resultados.iter().find(|r| r.chinchon).cloned();
    if let Some(resul_corto) = chinchon {
        (
            format!(
                "**{}**-sama hizo chinchon o.O\n{}",
                corto.name,
                lista_cartas(&resul_corto.juegos[0])
            ),
            vec![],
        )
    } else {
        let mut cont = format!("El sempaii **{}** acaba de cortar 😳 😳 😳\n\n", corto.name);
        for res in resultados {
            let _ = write!(
                cont,
                "Cartas de **{}**:\n{}\n\
                Suma {} y se queda en **{}**\n\n",
                res.jugador
                    .to_user(&http)
                    .await
                    .map(|u| u.name)
                    .unwrap_or_else(|_| "?".to_owned()),
                res.juegos
                    .iter()
                    .chain(iter::once(&res.sobrantes).filter(|cs| !cs.is_empty()))
                    .map(|cs| lista_cartas(cs))
                    .collect::<Vec<_>>()
                    .join("\n"),
                res.puntos_sumados,
                res.puntos_total
            );
        }
        let mut acciones = vec![];
        if let Some(turno) = prox_turno {
            cont += &format!("Ahora es el turno de <@{}> :3", turno);
            let mut row = CreateActionRow::default();
            row.create_button(|btn| btn.custom_id(format!("jugar {}", turno)).label("Jugar"));
            acciones.push(row)
        }
        (cont, acciones)
    }
}
