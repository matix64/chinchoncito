use super::{
    buscar_juegos::formar_juegos,
    cartas::{Carta, Palo},
};
use anyhow::{anyhow, Result};
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serenity::model::id::UserId;
use std::{
    collections::{HashMap, HashSet},
    mem::{swap, take},
    time::{Duration, SystemTime},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Partida {
    tiempo_ultima_jugada: SystemTime,
    mazo: Vec<Carta>,
    descartes: Vec<Carta>,
    pila_ultimo_levante: Option<PilaCartas>,
    jugadores: Vec<DatosJugador>,
    turno: usize,
    inicia_prox_ronda: usize,
}

impl Partida {
    pub fn empezar(jugadores: &[UserId]) -> Self {
        let mut s = Self {
            tiempo_ultima_jugada: SystemTime::now(),
            jugadores: jugadores
                .iter()
                .map(|id| DatosJugador {
                    id: *id,
                    ..Default::default()
                })
                .collect(),
            mazo: vec![],
            descartes: vec![],
            pila_ultimo_levante: None,
            turno: 0,
            inicia_prox_ronda: 0,
        };
        s.comenzar_ronda();
        s
    }

    fn comenzar_ronda(&mut self) {
        self.descartes.clear();
        self.mazo = mazo_mezclado();
        self.pila_ultimo_levante = None;
        for (i, jugador) in self.jugadores.iter_mut().enumerate() {
            jugador.mano = self
                .mazo
                .drain(0..if i == self.inicia_prox_ronda { 8 } else { 7 })
                .collect();
            jugador.mano.sort_unstable();
        }
        while self.jugadores[self.inicia_prox_ronda].perdio() {
            self.inicia_prox_ronda = (self.inicia_prox_ronda + 1) % self.jugadores.len();
        }
        self.turno = self.inicia_prox_ronda;
        self.inicia_prox_ronda = (self.inicia_prox_ronda + 1) % self.jugadores.len();
    }

    pub fn tiempo_inactiva(&self) -> Duration {
        self.tiempo_ultima_jugada
            .elapsed()
            .unwrap_or(Duration::ZERO)
    }

    fn pasar_turno(&mut self) {
        self.tiempo_ultima_jugada = SystemTime::now();
        loop {
            self.turno = (self.turno + 1) % self.jugadores.len();
            if !self.jugadores[self.turno].perdio() {
                break;
            }
        }
    }

    pub fn get_descarte(&self) -> Option<Carta> {
        self.descartes.last().cloned()
    }

    pub fn get_turno(&self) -> UserId {
        self.jugadores[self.turno].id
    }

    pub fn get_puntos(&self) -> HashMap<UserId, i16> {
        self.jugadores.iter().map(|j| (j.id, j.puntos)).collect()
    }

    pub fn get_pila_ultimo_levante(&self) -> Option<PilaCartas> {
        self.pila_ultimo_levante
    }

    pub fn jugadores_en_juego(&self) -> usize {
        self.jugadores.iter().filter(|j| !j.perdio()).count()
    }

    fn buscar_jugador(&mut self, id: UserId) -> Option<(usize, &mut DatosJugador)> {
        self.jugadores
            .iter_mut()
            .enumerate()
            .find(|(_, j)| j.id == id && !j.perdio())
    }

    pub fn ganador(&self) -> Option<UserId> {
        let mut no_perdieron = self.jugadores.iter().filter(|j| !j.perdio());
        let posible_ganador = no_perdieron.next();
        if no_perdieron.next().is_none() {
            posible_ganador.map(|j| j.id)
        } else {
            None
        }
    }

    pub fn jugador(&mut self, id: UserId) -> Option<Jugador<'_>> {
        let (indice, _) = self.buscar_jugador(id)?;
        Some(Jugador {
            indice,
            partida: self,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DatosJugador {
    id: UserId,
    mano: Vec<Carta>,
    puntos: i16,
    #[serde(skip)]
    #[serde(default = "HashSet::new")]
    votos_expulsar: HashSet<UserId>,
    eliminado: bool,
}

impl DatosJugador {
    const fn perdio(&self) -> bool {
        self.puntos > 100 || self.eliminado
    }

    const fn pierde_sumando(&self, suma: i16) -> bool {
        (self.puntos + suma) > 100 || self.eliminado
    }

    fn votar_expulsar(&mut self, jugador: UserId, total_jugadores: usize) -> bool {
        self.votos_expulsar.insert(jugador);
        self.eliminado = self.votos_expulsar.len() > total_jugadores / 2;
        self.eliminado
    }

    fn abandonar(&mut self) {
        self.eliminado = true;
    }
}

#[derive(Debug)]
pub struct Jugador<'a> {
    indice: usize,
    pub partida: &'a mut Partida,
}

impl<'a> Jugador<'a> {
    fn datos(&self) -> &DatosJugador {
        &self.partida.jugadores[self.indice]
    }

    pub fn es_turno(&self) -> bool {
        self.partida.turno == self.indice
    }

    pub fn get_cartas(&self) -> Vec<Carta> {
        self.datos().mano.clone()
    }

    pub fn puede_cortar(&self, con: Carta) -> bool {
        let mut mano = self.datos().mano.clone();
        if let Ok(i) = mano.binary_search(&con) {
            mano.remove(i);
        }
        let (puntos_sumados, _) = formar_juegos(mano);
        puntos_sumados <= 5 && !self.datos().pierde_sumando(puntos_sumados)
    }

    fn datos_mut(&mut self) -> &mut DatosJugador {
        &mut self.partida.jugadores[self.indice]
    }

    pub fn tirar(&mut self, carta: Carta) -> Result<(), ErrorTirar> {
        if !self.es_turno() {
            return Err(ErrorTirar::NoEsTurno);
        }
        let mano = &mut self.datos_mut().mano;
        if mano.len() < 8 {
            return Err(ErrorTirar::DebeLevantar);
        }
        let ind_carta = mano
            .binary_search(&carta)
            .map_err(|_| ErrorTirar::NoTieneCarta)?;
        mano.remove(ind_carta);
        self.partida.descartes.push(carta);
        self.partida.pasar_turno();
        Ok(())
    }

    pub fn levantar(&mut self, pila: PilaCartas) -> Result<Carta, ErrorLevantar> {
        if !self.es_turno() {
            return Err(ErrorLevantar::NoEsTurno);
        }
        let Partida {
            mazo,
            descartes,
            jugadores,
            pila_ultimo_levante,
            ..
        } = self.partida;
        let mano = &mut jugadores[self.indice].mano;
        if mano.len() >= 8 {
            return Err(ErrorLevantar::DebeBajar);
        }
        let carta = if pila == PilaCartas::Mazo {
            if mazo.is_empty() {
                swap(mazo, descartes);
                descartes.push(mazo.pop().unwrap());
                mazo.shuffle(&mut thread_rng());
            }
            Ok(mazo.pop().unwrap())
        } else {
            descartes.pop().ok_or(ErrorLevantar::NoHayDescartes)
        }?;
        let pos = mano.binary_search(&carta).unwrap_or_else(|e| e);
        mano.insert(pos, carta);
        *pila_ultimo_levante = Some(pila);
        Ok(carta)
    }

    pub fn cortar(
        &mut self,
        carta: Option<Carta>,
    ) -> Result<Vec<ResultadoFinalRonda>, ErrorCortar> {
        if !self.es_turno() {
            return Err(ErrorCortar::NoEsTurno);
        }
        if let Some(carta) = carta {
            if self.datos().mano.len() < 8 {
                return Err(ErrorCortar::NoPuedeBajar);
            }
            let ind_carta = self
                .datos()
                .mano
                .binary_search(&carta)
                .map_err(|_| ErrorCortar::NoTieneCarta)?;
            self.datos_mut().mano.remove(ind_carta);
        } else if self.datos().mano.len() >= 8 {
            return Err(ErrorCortar::DebeBajar);
        }
        let mut resultados: Vec<_> = self
            .partida
            .jugadores
            .iter()
            .enumerate()
            .map(|(i, j)| {
                if j.perdio() {
                    return None;
                }
                let (mut puntos_sumados, juegos) = formar_juegos(j.mano.clone());
                let mut chinchon = false;
                if i == self.indice && puntos_sumados == 0 {
                    puntos_sumados = -10;
                    if juegos.len() == 1 {
                        chinchon = true;
                    }
                }
                Some(ResultadoFinalRonda {
                    jugador: j.id,
                    puntos_sumados,
                    puntos_total: (j.puntos + puntos_sumados).max(0),
                    perdio: j.pierde_sumando(puntos_sumados),
                    chinchon,
                    sobrantes: j
                        .mano
                        .iter()
                        .filter(|c| juegos.iter().flatten().all(|cj| cj != *c))
                        .copied()
                        .collect(),
                    juegos,
                })
            })
            .collect();
        let resul_propio = resultados[self.indice].clone().unwrap();
        if resul_propio.puntos_sumados > 5 || resul_propio.perdio {
            if let Some(carta) = carta {
                let pos = self
                    .datos()
                    .mano
                    .binary_search(&carta)
                    .unwrap_or_else(|e| e);
                self.datos_mut().mano.insert(pos, carta);
            }
            return Err(ErrorCortar::PuntajeMuyAlto);
        } else if resul_propio.chinchon {
            for resul in resultados.iter_mut().flat_map(|r| r.as_mut()) {
                if resul.jugador != resul_propio.jugador {
                    resul.puntos_sumados += 1000;
                    resul.puntos_total += 1000;
                    resul.perdio = true;
                }
            }
        }
        for (jugador, resultado) in self.partida.jugadores.iter_mut().zip(resultados.iter()) {
            if let Some(resultado) = resultado {
                jugador.puntos = resultado.puntos_total;
            }
        }
        self.partida.comenzar_ronda();
        let len = resultados.len();
        Ok(resultados
            .into_iter()
            .flatten()
            .cycle()
            .skip(self.indice)
            .take(len)
            .collect())
    }

    pub fn votar_expulsar_a(&mut self, a: UserId) -> Result<bool> {
        let restantes = self.partida.jugadores_en_juego();
        let id_propio = self.datos().id;
        let (turno_victima, victima) = self
            .partida
            .buscar_jugador(a)
            .ok_or_else(|| anyhow!("Ese compa no esta en la partida :/"))?;
        let expulsado = victima.votar_expulsar(id_propio, restantes);
        if expulsado {
            let cartas = take(&mut victima.mano);
            self.partida.mazo.extend(cartas);
            self.partida.mazo.shuffle(&mut thread_rng());
            if self.partida.turno == turno_victima {
                self.partida.pasar_turno();
            }
        }
        Ok(expulsado)
    }

    pub fn abandonar(&mut self) {
        let datos = self.datos_mut();
        datos.abandonar();
        let cartas = take(&mut datos.mano);
        self.partida.mazo.extend(cartas);
        self.partida.mazo.shuffle(&mut thread_rng());
        if self.es_turno() {
            self.partida.pasar_turno();
        }
    }
}

fn mazo_mezclado() -> Vec<Carta> {
    let mut cartas: Vec<Carta> = (1..=12)
        .flat_map(|num| {
            [Palo::Copa, Palo::Espada, Palo::Oro, Palo::Basto]
                .into_iter()
                .map(move |palo| Carta { num, palo })
        })
        .collect();
    cartas.shuffle(&mut thread_rng());
    cartas
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum PilaCartas {
    Mazo,
    Descartes,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ErrorTirar {
    NoEsTurno,
    DebeLevantar,
    NoTieneCarta,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ErrorLevantar {
    NoEsTurno,
    DebeBajar,
    NoHayDescartes,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ErrorCortar {
    NoEsTurno,
    PuntajeMuyAlto,
    NoTieneCarta,
    NoPuedeBajar,
    DebeBajar,
}

#[derive(Debug, Clone)]
pub struct ResultadoFinalRonda {
    pub jugador: UserId,
    pub puntos_sumados: i16,
    pub puntos_total: i16,
    pub perdio: bool,
    pub chinchon: bool,
    pub juegos: Vec<Vec<Carta>>,
    pub sobrantes: Vec<Carta>,
}
