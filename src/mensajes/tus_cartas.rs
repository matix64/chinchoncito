use super::lista_cartas;
use crate::chinchon::Carta;

pub fn mensaje_cartas(cartas: &[Carta]) -> String {
    format!("Tus cartas: {}", lista_cartas(cartas))
}
