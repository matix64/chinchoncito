use super::cartas::Carta;

pub fn formar_juegos(mut cartas: Vec<Carta>) -> (i16, Vec<Vec<Carta>>) {
    let mut juegos = vec![];
    cartas.sort_unstable_by(|a, b| a.palo.cmp(&b.palo).then(a.num.cmp(&b.num)));
    for tamaño in 3..8 {
        for grupo in cartas.windows(tamaño) {
            if grupo
                .windows(2)
                .all(|w| w[0].palo == w[1].palo && w[0].num + 1 == w[1].num)
            {
                juegos.push(grupo.to_owned());
            }
        }
    }
    cartas.sort_unstable_by_key(|c| c.num);
    for grupo in cartas.windows(4) {
        if grupo[1..].iter().all(|c| c.num == grupo[0].num) {
            juegos.push(grupo.to_owned());
            juegos.push(vec![grupo[0], grupo[2], grupo[3]]);
            juegos.push(vec![grupo[0], grupo[1], grupo[3]]);
        }
    }
    for grupo in cartas.windows(3) {
        if grupo[1..].iter().all(|c| c.num == grupo[0].num) {
            juegos.push(grupo.to_owned());
        }
    }
    let (puntos_juegos, mej_juegos) =
        mejor_combinacion_juegos(&juegos.iter().map(|j| j.as_slice()).collect::<Vec<_>>());
    let suma_cartas: u8 = cartas.iter().map(|c| c.num).sum();
    (
        suma_cartas as i16 - puntos_juegos as i16,
        mej_juegos.into_iter().map(|j| j.to_owned()).collect(),
    )
}

fn mejor_combinacion_juegos<'a>(juegos_posibles: &[&'a [Carta]]) -> (i8, Vec<&'a [Carta]>) {
    let mut mej_puntaje = 0;
    let mut mej_juegos = vec![];
    for juego in juegos_posibles {
        let juegos_no_solapados: Vec<_> = juegos_posibles
            .iter()
            .filter(|j| !j.iter().any(|c| juego.contains(c)))
            .copied()
            .collect();
        let (mut puntaje, mut juegos) = mejor_combinacion_juegos(&juegos_no_solapados);
        puntaje += juego.iter().map(|c| c.num as i8).sum::<i8>();
        if puntaje > mej_puntaje {
            mej_puntaje = puntaje;
            juegos.push(juego);
            mej_juegos = juegos;
        } else if puntaje == mej_puntaje && juegos.len() < mej_juegos.len() {
            juegos.push(juego);
            mej_juegos = juegos;
        }
    }
    (mej_puntaje, mej_juegos)
}
