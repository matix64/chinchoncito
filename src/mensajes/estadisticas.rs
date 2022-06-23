use crate::estadisticas::{Estadisticas, EstadisticasJugador};
use anyhow::Result;
use serenity::{
    builder::CreateEmbed,
    model::{id::GuildId, user::User},
};

pub async fn mensaje_estadisticas(
    jugador: &User,
    server: Option<GuildId>,
    estadisticas: &Estadisticas,
) -> Result<CreateEmbed> {
    let stats_globales = estadisticas.get(jugador.id, None).await?;
    let mut mensaje = seccion_tabla("Globales", &stats_globales);
    if server.is_some() {
        let stats_server = estadisticas.get(jugador.id, server).await?;
        mensaje = mensaje + "\n\n" + &seccion_tabla("Este servidor", &stats_server);
    }
    let mut embed = CreateEmbed::default();
    embed
        .title(format!("Estadisticas de {}", jugador.name))
        .description(mensaje);
    Ok(embed)
}

fn seccion_tabla(nombre: &str, stats: &EstadisticasJugador) -> String {
    let EstadisticasJugador {
        victorias,
        derrotas,
    } = stats;
    let total = victorias + derrotas;
    let mut texto = format!("**{nombre}:**\n**Partidas:** {total}");
    if total > 0 {
        let porcentaje = victorias * 100 / total;
        texto += &format!("\n**Ganadas:** {victorias} ({porcentaje} %)");
    }
    texto
}
