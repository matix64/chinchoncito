use rand::{prelude::SliceRandom, thread_rng};
use serenity::{
    builder::CreateEmbed,
    http::CacheHttp,
    model::id::{GuildId, UserId},
};

const IMAGENES: &[&str] = &[
    "https://media.giphy.com/media/Diym3aZO1dHzO/giphy.gif",
    "https://media.giphy.com/media/klQrJUcrfMsTK/giphy.gif",
];

pub async fn mensaje_fin_partida(
    http: &impl CacheHttp,
    server: GuildId,
    ganador: UserId,
) -> CreateEmbed {
    let nombre = match server.member(http, ganador).await.ok().and_then(|m| m.nick) {
        Some(nick) => nick,
        None => ganador
            .to_user(http)
            .await
            .map(|u| u.name)
            .unwrap_or_else(|_| "?".to_owned()),
    };
    let mut embed = CreateEmbed::default();
    embed
        .title(format!("Felicidades {nombre}"))
        .description("Ganaste sisi :D")
        .image(IMAGENES.choose(&mut thread_rng()).unwrap());
    embed
}
