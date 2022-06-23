use crate::chinchon::Partida;
use anyhow::{anyhow, Result};
use rmp_serde::{encode::write_named, from_read};
use serenity::model::id::{ChannelId, MessageId, UserId};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    future::Future,
    io,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{Mutex, RwLock},
    task::spawn_blocking,
};

#[derive(Default)]
pub struct ListaPartidas {
    invitaciones: RwLock<HashMap<(ChannelId, UserId), (Invitacion, MessageId)>>,
    partidas: RwLock<HashMap<ChannelId, Arc<Mutex<Partida>>>>,
}

impl ListaPartidas {
    pub async fn cargar() -> Result<Self> {
        let partidas = spawn_blocking::<_, Result<_>>(|| {
            let partidas: Vec<(ChannelId, Partida)> = match File::open("partidas") {
                Ok(arch) => from_read(arch)?,
                Err(e) => match e.kind() {
                    io::ErrorKind::NotFound => vec![],
                    _ => Err(e)?,
                },
            };
            Ok(partidas
                .into_iter()
                .map(|(c, p)| (c, Arc::new(Mutex::new(p))))
                .collect())
        })
        .await
        .unwrap()?;
        Ok(Self {
            partidas: RwLock::new(partidas),
            ..Default::default()
        })
    }

    pub async fn guardar(&self) -> Result<()> {
        let partidas = self.partidas.read().await;
        let mut lista = Vec::with_capacity(partidas.len());
        for (canal, partida) in partidas.iter() {
            let partida = partida.lock().await.clone();
            if partida.tiempo_inactiva() < Duration::from_secs(24 * 60 * 60) {
                lista.push((*canal, partida));
            }
        }
        drop(partidas);
        spawn_blocking(move || {
            let mut arch = File::create("partidas")?;
            write_named(&mut arch, &lista)?;
            Ok(())
        })
        .await
        .unwrap()
    }

    pub async fn crear_invitacion(
        &self,
        canal: ChannelId,
        creador: UserId,
        mensaje: MessageId,
        invitados: Option<Vec<UserId>>,
        max_jugadores: usize,
    ) -> Option<(ChannelId, MessageId)> {
        let invitacion_vieja = self.invitaciones.write().await.insert(
            (canal, creador),
            (Invitacion::new(creador, invitados, max_jugadores), mensaje),
        );
        invitacion_vieja.map(|i| (canal, i.1))
    }

    pub async fn agregar_invitado(
        &self,
        canal: ChannelId,
        creador: UserId,
        invitado: UserId,
    ) -> Result<()> {
        self.invitaciones
            .write()
            .await
            .get_mut(&(canal, creador))
            .ok_or_else(|| anyhow!(
                "No creaste ninguna invitacion, usa **/chinchon** para crear una"
            ))?
            .0
            .agregar_invitado(invitado)
    }

    pub async fn aceptar_invitacion(
        &self,
        canal: ChannelId,
        creador: UserId,
        mensaje: MessageId,
        acepta: UserId,
    ) -> Result<Invitacion> {
        let mut invitaciones = self.invitaciones.write().await;
        let (invi, mensaje_invi) = invitaciones
            .get_mut(&(canal, creador))
            .ok_or_else(|| anyhow!("bb esta invitacion ya no existe u.u"))?;
        if *mensaje_invi != mensaje {
            return Err(anyhow!("bb esta invitacion ya no existe :c"));
        }
        invi.aceptar(acepta)?;
        Ok(invi.clone())
    }

    pub async fn empezar_partida<Fut>(
        &self,
        canal_inv: ChannelId,
        creador_inv: UserId,
        crear_canal: impl FnOnce(MessageId, Vec<UserId>, UserId) -> Fut,
    ) -> std::result::Result<RespuestaEmpezarPartida, ErrorEmpezarPartida>
    where
        Fut: Future<Output = Result<ChannelId>>,
    {
        let mut invitaciones = self.invitaciones.write().await;
        let (invitacion, mensaje_invi) = invitaciones
            .get(&(canal_inv, creador_inv))
            .ok_or(ErrorEmpezarPartida::InvitacionNoExiste)?
            .clone();
        if invitacion.jugadores().len() < 2 {
            return Err(ErrorEmpezarPartida::PocosJugadores);
        }
        invitaciones.remove(&(canal_inv, creador_inv)).unwrap();
        drop(invitaciones);
        let jugadores = invitacion.jugadores();
        let partida = Partida::empezar(&jugadores);
        let comienza = partida.get_turno();
        let canal_partida = crear_canal(mensaje_invi, jugadores, comienza)
            .await
            .map_err(|_| ErrorEmpezarPartida::ErrorCreandoCanal)?;
        let partida = Arc::new(Mutex::new(partida));
        let mut partidas = self.partidas.write().await;
        partidas.insert(canal_partida, partida.clone());
        Ok(RespuestaEmpezarPartida {
            invitacion,
            mensaje_invi,
            partida,
        })
    }

    pub async fn get_partida(&self, canal: ChannelId) -> Option<Arc<Mutex<Partida>>> {
        self.partidas.read().await.get(&canal).cloned()
    }

    pub async fn terminar_partida(&self, canal: ChannelId) -> Result<()> {
        let mut partidas = self.partidas.write().await;
        match partidas.remove(&canal) {
            Some(_) => Ok(()),
            _ => Err(anyhow!("La partida no existe")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Invitacion {
    invitados: Option<HashSet<UserId>>,
    aceptaron: HashSet<UserId>,
    pub max_jugadores: usize,
}

impl Invitacion {
    fn new(invita: UserId, invitados: Option<Vec<UserId>>, max_jugadores: usize) -> Self {
        Self {
            invitados: invitados.map(|invs| invs.into_iter().collect()),
            aceptaron: [invita].into_iter().collect(),
            max_jugadores,
        }
    }

    pub fn jugadores(&self) -> Vec<UserId> {
        self.aceptaron.iter().cloned().collect()
    }

    pub fn llena(&self) -> bool {
        self.aceptaron.len() >= self.max_jugadores
    }

    pub fn privada(&self) -> bool {
        self.invitados.is_some()
    }

    fn aceptar(&mut self, acepta: UserId) -> Result<()> {
        if let Some(ref invitados) = self.invitados {
            if !invitados.contains(&acepta) {
                return Err(anyhow!("Perdon sempaii pero no te invitaron :("));
            }
        }
        if self.aceptaron.insert(acepta) {
            Ok(())
        } else {
            Err(anyhow!("Pero-pero ya estas en esta partida onii-chan"))
        }
    }

    fn agregar_invitado(&mut self, invitado: UserId) -> Result<()> {
        self.invitados
            .as_mut()
            .ok_or_else(|| anyhow!("Tu mesa es publica"))?
            .insert(invitado);
        Ok(())
    }
}

pub struct RespuestaEmpezarPartida {
    pub invitacion: Invitacion,
    pub mensaje_invi: MessageId,
    pub partida: Arc<Mutex<Partida>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorEmpezarPartida {
    InvitacionNoExiste,
    PocosJugadores,
    ErrorCreandoCanal,
}
