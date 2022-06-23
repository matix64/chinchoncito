use anyhow::{anyhow, Result};
use serenity::model::{
    channel::PartialChannel,
    interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
    user::User,
};

pub fn get_opcion_o_none<T>(
    nombre: &str,
    inter: &ApplicationCommandInteraction,
) -> Result<Option<T>>
where
    ApplicationCommandInteractionDataOptionValue: ValorOpcion<T>,
{
    inter
        .data
        .options
        .iter()
        .find_map(|o| {
            if o.name == nombre {
                o.resolved.as_ref()
            } else {
                None
            }
        })
        .map(|v| v.valor_opcion())
        .transpose()
}

pub fn get_opcion_o_default<T>(
    nombre: &str,
    inter: &ApplicationCommandInteraction,
    def: T,
) -> Result<T>
where
    ApplicationCommandInteractionDataOptionValue: ValorOpcion<T>,
{
    Ok(get_opcion_o_none(nombre, inter)?.unwrap_or(def))
}

pub fn get_opcion<T>(nombre: &str, inter: &ApplicationCommandInteraction) -> Result<T>
where
    ApplicationCommandInteractionDataOptionValue: ValorOpcion<T>,
{
    get_opcion_o_none(nombre, inter)?.ok_or_else(|| anyhow!("opcion {} no encontrada", nombre))
}

pub trait ValorOpcion<T> {
    fn valor_opcion(&self) -> Result<T>;
}

impl ValorOpcion<String> for ApplicationCommandInteractionDataOptionValue {
    fn valor_opcion(&self) -> Result<String> {
        match self {
            ApplicationCommandInteractionDataOptionValue::String(s) => Ok(s.clone()),
            _ => Err(anyhow!("tipo invalido")),
        }
    }
}

impl ValorOpcion<i64> for ApplicationCommandInteractionDataOptionValue {
    fn valor_opcion(&self) -> Result<i64> {
        match self {
            ApplicationCommandInteractionDataOptionValue::Integer(i) => Ok(*i),
            _ => Err(anyhow!("tipo invalido")),
        }
    }
}

impl ValorOpcion<bool> for ApplicationCommandInteractionDataOptionValue {
    fn valor_opcion(&self) -> Result<bool> {
        match self {
            ApplicationCommandInteractionDataOptionValue::Boolean(b) => Ok(*b),
            ApplicationCommandInteractionDataOptionValue::String(s) => Ok(s.parse()?),
            _ => Err(anyhow!("tipo invalido")),
        }
    }
}

impl ValorOpcion<User> for ApplicationCommandInteractionDataOptionValue {
    fn valor_opcion(&self) -> Result<User> {
        match self {
            ApplicationCommandInteractionDataOptionValue::User(u, ..) => Ok(u.clone()),
            _ => Err(anyhow!("tipo invalido")),
        }
    }
}

impl ValorOpcion<PartialChannel> for ApplicationCommandInteractionDataOptionValue {
    fn valor_opcion(&self) -> Result<PartialChannel> {
        match self {
            ApplicationCommandInteractionDataOptionValue::Channel(c) => Ok(c.clone()),
            _ => Err(anyhow!("tipo invalido")),
        }
    }
}
