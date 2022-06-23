use anyhow::{anyhow, Result};
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::model::id::EmojiId;
use std::{fmt::Display, str::FromStr};

static EMOJIS_PALOS: OnceCell<[EmojiId; 4]> = OnceCell::new();
static COD_CARTA_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d{1,2}) ?(?:de? )?([a-z]+)").unwrap());

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Carta {
    pub num: u8,
    pub palo: Palo,
}

impl Carta {
    pub fn nombre(&self) -> String {
        format!("{} de {}", self.num, self.palo.nombre())
    }
}

impl Display for Carta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "**{}**\u{202f}{}", self.num, self.palo)
    }
}

impl FromStr for Carta {
    type Err = CartaFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match COD_CARTA_RE.captures(&s.to_ascii_lowercase()) {
            Some(capturas) => {
                let num = capturas[1].parse().unwrap();
                if num == 0 || num > 12 {
                    return Err(Self::Err::NumeroInvalido(num));
                }
                Ok(Self {
                    num,
                    palo: capturas[2]
                        .parse()
                        .map_err(|e: PaloFromStrError| Self::Err::PaloInvalido(e.string))?,
                })
            }
            None => Err(Self::Err::CodigoNoEncontrado),
        }
    }
}

pub enum CartaFromStrErr {
    CodigoNoEncontrado,
    PaloInvalido(String),
    NumeroInvalido(u8),
}

pub fn inicializar_emojis_palos(
    copa: EmojiId,
    espada: EmojiId,
    oro: EmojiId,
    basto: EmojiId,
) -> Result<()> {
    EMOJIS_PALOS
        .set([copa, espada, oro, basto])
        .map_err(|_| anyhow!("No se puede inicializar_emojis_palos 2 veces"))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Palo {
    Copa = 0,
    Espada,
    Oro,
    Basto,
}

impl Palo {
    pub const fn nombre(&self) -> &'static str {
        match self {
            Self::Copa => "copas",
            Self::Espada => "espada",
            Self::Oro => "oro",
            Self::Basto => "basto",
        }
    }

    pub fn emoji(&self) -> EmojiId {
        EMOJIS_PALOS
            .get()
            .map(|ids| ids[*self as usize])
            .unwrap_or_default()
    }
}

impl Display for Palo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<:{}:{}>", self.nombre(), self.emoji().as_u64())
    }
}

impl FromStr for Palo {
    type Err = PaloFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "o" | "oro" => Ok(Self::Oro),
            "e" | "esp" | "espada" | "espadas" => Ok(Self::Espada),
            "c" | "copa" | "copas" => Ok(Self::Copa),
            "b" | "basto" | "bastos" | "p" | "palo" | "palos" => Ok(Self::Basto),
            _ => Err(PaloFromStrError {
                string: s.to_owned(),
            }),
        }
    }
}

pub struct PaloFromStrError {
    pub string: String,
}
