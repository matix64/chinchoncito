mod cortar;
mod estadisticas;
mod fin_partida;
mod invitacion;
mod jugar;
mod tus_cartas;

pub use cortar::mensaje_cortar;
pub use estadisticas::mensaje_estadisticas;
pub use fin_partida::mensaje_fin_partida;
pub use invitacion::mensaje_invitacion;
pub use jugar::mensaje_jugar;
pub use tus_cartas::mensaje_cartas;

use crate::chinchon::Carta;

fn lista_cartas(cartas: &[Carta]) -> String {
    let lista = cartas
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" │ ");
    format!("│ {} │", lista)
}
