use super::lista_cartas;
use crate::chinchon::{Carta, Jugador};
use serenity::{builder::CreateActionRow, model::interactions::message_component::ButtonStyle};

pub fn mensaje_jugar(
    jugador: &Jugador<'_>,
    levantada: Option<Carta>,
    seleccionada: Option<Carta>,
) -> (String, Vec<CreateActionRow>) {
    let cartas = jugador.get_cartas();
    if cartas.len() == 8 {
        let texto = format!(
            "{}Tus cartas son:\n{}",
            levantada
                .map(|c| format!("Levantaste un {}\n", c))
                .unwrap_or_default(),
            lista_cartas(&cartas)
        );
        let mut componentes = vec![CreateActionRow::default(); 1];
        componentes[0].create_select_menu(|sel| {
            sel.custom_id("selec carta")
                .placeholder("Elige una carta para bajar o cortar")
                .options(|opts| {
                    for carta in cartas {
                        opts.create_option(|opt| {
                            opt.label(carta.num.to_string())
                                .value(carta.nombre())
                                .emoji(carta.palo.emoji())
                                .default_selection(Some(carta) == seleccionada)
                        });
                    }
                    opts
                })
        });
        if let Some(carta) = seleccionada {
            componentes.push(Default::default());
            componentes[1].create_button(|btn| {
                btn.custom_id(format!("bajar {}", carta.nombre()))
                    .label("Bajar")
            });
            componentes[1].create_button(|btn| {
                btn.custom_id(format!("cortar {}", carta.nombre()))
                    .label("Cortar")
                    .style(ButtonStyle::Danger)
            });
        }
        (texto, componentes)
    } else {
        let descarte = jugador.partida.get_descarte();
        let texto = format!(
            "Tus cartas son:\n{}\n{}",
            lista_cartas(&cartas),
            descarte
                .map(|c| format!("El ultimo descarte es {}", c))
                .unwrap_or_else(|| "No hay descartes :(".to_owned())
        );
        let mut componentes = vec![CreateActionRow::default(); 1];
        componentes[0]
            .create_button(|btn| btn.custom_id("levantar mazo").label("Levantar del mazo"));
        if let Some(carta) = descarte {
            componentes[0].create_button(|btn| {
                btn.custom_id("levantar descarte")
                    .label(format!("Llevarse el {}", carta.nombre()))
            });
        }
        (texto, componentes)
    }
}
