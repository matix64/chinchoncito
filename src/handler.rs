use crate::{
    comandos::procesar_comando, componentes::inter_componente, config_servers::ConfigServers,
    estadisticas::Estadisticas, lista_partidas::ListaPartidas,
};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::{ChannelType, Message},
        gateway::Ready,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction,
        },
        Permissions,
    },
};
use std::{mem::forget, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Handler(Arc<HandlerInner>);

struct HandlerInner {
    partidas: Arc<ListaPartidas>,
    config_servers: ConfigServers,
    estadisticas: Estadisticas,
    comandos_en_proceso: RwLock<()>,
}

impl Handler {
    pub fn new(
        partidas: Arc<ListaPartidas>,
        config_servers: ConfigServers,
        estadisticas: Estadisticas,
    ) -> Self {
        Self(Arc::new(HandlerInner {
            partidas,
            config_servers,
            estadisticas,
            comandos_en_proceso: RwLock::new(()),
        }))
    }

    pub async fn detener(&self) {
        forget(self.0.comandos_en_proceso.write().await);
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.is_private() && !msg.author.bot {
            let _ = msg.reply(&ctx.http, "uwu").await;
        }
    }

    async fn interaction_create(&self, ctx: Context, inter: Interaction) {
        let _lock_comando_en_proceso = match self.0.comandos_en_proceso.try_read() {
            Ok(lock) => lock,
            Err(_) => return,
        };
        match inter {
            Interaction::ApplicationCommand(inter) => {
                if let Err(err) = procesar_comando(
                    &ctx,
                    &inter,
                    &self.0.partidas,
                    &mut self.0.config_servers.clone(),
                    &mut self.0.estadisticas.clone(),
                )
                .await
                {
                    let _ = inter
                        .create_interaction_response(&ctx.http, |resp| {
                            resp.interaction_response_data(|msg| {
                                msg.ephemeral(true).content(err.to_string())
                            })
                        })
                        .await;
                }
            }
            Interaction::MessageComponent(mut inter) => {
                if let Err(err) = inter_componente(
                    &ctx,
                    &mut inter,
                    &self.0.partidas,
                    &mut self.0.estadisticas.clone(),
                )
                .await
                {
                    let _ = inter
                        .create_interaction_response(&ctx.http, |resp| {
                            resp.interaction_response_data(|msg| {
                                msg.ephemeral(true).content(err.to_string())
                            })
                        })
                        .await;
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        println!("Conectado!");
        ApplicationCommand::set_global_application_commands(&ctx.http, |x| {
            x.create_application_command(|c| {
                c.name("chinchon")
                    .description("Empezar una partida")
                    .dm_permission(false)
                    .create_option(|opt| {
                        opt.name("privada")
                            .description("Si quieres elegir quien puede unirse con /invitar")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("Sí", "true")
                            .add_string_choice("No", "false")
                    })
                    .create_option(|opt| {
                        opt.name("jugadores")
                            .description("El limite de jugadores. Por defecto es 2, maximo 4")
                            .kind(ApplicationCommandOptionType::Integer)
                    })
            })
            .create_application_command(|c| {
                c.name("stats")
                    .description("Ver las estadisticas de juego de alguien")
                    .create_option(|opt| {
                        opt.name("jugador")
                            .description("De quien quieres ver las estadisticas. Omitelo para ver las tuyas")
                            .kind(ApplicationCommandOptionType::User)
                            .required(false)
                    })
            })
            .create_application_command(|c| {
                c.name("invitar")
                    .description("Autorizar a alguien a unirse a tu partida. Solo funciona en mesas privadas")
                    .dm_permission(false)
                    .create_option(|opt| {
                       opt.name("a")
                           .description("A quien quieres invitar")
                           .kind(ApplicationCommandOptionType::User)
                           .required(true)
                    })
            })
            .create_application_command(|c| {
                c.name("cartas")
                    .description("Ver tus cartas")
                    .dm_permission(false)
            })
            .create_application_command(|c| {
                c.name("jugar")
                    .description("Usalo para jugar cuando sea tu turno")
                    .dm_permission(false)
            })
            .create_application_command(|c| {
                c.name("puntos")
                    .description("Ver los puntajes de la partida en la que estas")
                    .dm_permission(false)
            })
            .create_application_command(|c| {
                c.name("empezar")
                    .description("Si creaste una partida y todavia no se llena usa este comando para empezarla igual")
                    .dm_permission(false)
            })
            .create_application_command(|c| {
                c.name("canal")
                    .description("Elegir el canal donde se pueden crear partidas")
                    .dm_permission(false)
                    .default_member_permissions(Permissions::MANAGE_CHANNELS)
                    .create_option(|o| {
                        o.name("canal")
                            .description("El canal para crear partidas")
                            .kind(ApplicationCommandOptionType::Channel)
                            .channel_types(&[ChannelType::Text])
                            .required(true)
                    })
            })
            .create_application_command(|c| {
                c.name("kick")
                    .description("Vota para expulsar a alguien de una partida")
                    .dm_permission(false)
                    .create_option(|o| {
                        o.name("a")
                            .description("A quien quieres expulsar")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
            })
            .create_application_command(|c| {
                c.name("salir")
                    .description("Abandonar la partida")
                    .dm_permission(false)
            })
        })
        .await
        .expect("Error creando comandos");
    }
}
