use std::fmt;
use crate::prelude::*;
use serde::de::{Deserialize, SeqAccess, Visitor, Error};

// since replays are cached
// might want to optimize for space

// u16 can hold up to 2^16 = 65535. 50 * 50 (max size of map) < 65535
type TileIndex = u16;

type PlayerIndex = u8;

type Turn = u32;

#[derive(Clone, Default)]
pub struct Options {
    pub speed: f32,
    pub city_density: f32,
    pub mountain_density: f32,
    pub swamp_density: f32
}

impl<'de> Deserialize<'de> for Options {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: serde::Deserializer<'de> {

        struct OptionsVisitor;
        impl<'de> Visitor<'de> for OptionsVisitor {
            type Value = Options;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("options struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                    A: serde::de::SeqAccess<'de>, {
                let mut options: Options = Default::default();
                options.speed = seq.next_element::<f32>()?.ok_or(A::Error::custom("options missing speed"))?;
                options.city_density = seq.next_element::<f32>()?.ok_or(A::Error::custom("options missing city_density"))?;
                options.mountain_density = seq.next_element::<f32>()?.ok_or(A::Error::custom("options missing mountain_density"))?;
                options.swamp_density = seq.next_element::<f32>()?.ok_or(A::Error::custom("options missing swamp_density"))?;
                Ok(options)
            }
        }

        deserializer.deserialize_seq(OptionsVisitor)

    }
}

#[derive(Clone, Default)]
pub struct Move {
    pub player_index: PlayerIndex,
    pub start: TileIndex,
    pub end: TileIndex,
    pub is50: bool,
    pub turn: Turn,
}

impl<'de> Deserialize<'de> for Move {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: serde::Deserializer<'de> {

        struct MoveVisitor;
        impl<'de> Visitor<'de> for MoveVisitor {
            type Value = Move;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("move struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                    A: serde::de::SeqAccess<'de>, {
                let mut data: Move = Default::default();
                data.player_index = seq.next_element()?.ok_or(A::Error::custom("missing player_index"))?;
                data.start = seq.next_element()?.ok_or(A::Error::custom("missing start"))?;
                data.start = seq.next_element::<TileIndex>()?.ok_or(A::Error::custom("missing end"))?;
                data.is50 = seq.next_element::<bool>()?.ok_or(A::Error::custom("missing is50"))?;
                data.turn = seq.next_element::<Turn>()?.ok_or(A::Error::custom("missing turn"))?;
                Ok(data)
            }
        }

        deserializer.deserialize_seq(MoveVisitor)
    
    }
}

#[derive(Clone, Default)]
pub struct Surrender {
    pub index: usize,
    pub turn: Turn,
}

impl<'de> Deserialize<'de> for Surrender {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: serde::Deserializer<'de> {
        struct SurrenderVisitor;
        impl<'de> Visitor<'de> for SurrenderVisitor {
            type Value = Surrender;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("surrender struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                    A: serde::de::SeqAccess<'de>, {
                let mut data: Surrender = Default::default();
                data.index = seq.next_element()?.ok_or(A::Error::custom("missing index"))?;
                data.turn = seq.next_element()?.ok_or(A::Error::custom("missing turn"))?;
                Ok(data)
            }
        }

        deserializer.deserialize_seq(SurrenderVisitor)
    
    }
}

#[derive(Clone, Default)]
pub struct ChatMessage {
    pub message: String,
    pub prefix: String,
    pub player_index: PlayerIndex,
    pub turn: Turn,
}

impl<'de> Deserialize <'de> for ChatMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: serde::Deserializer<'de> {
        struct ChatVisitor;
        impl<'de> Visitor<'de> for ChatVisitor {
            type Value = ChatMessage;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "chat message")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                    A: SeqAccess<'de>, {
                let mut data: ChatMessage = Default::default();
                data.message = seq.next_element()?.ok_or(A::Error::custom("missing message"))?;
                data.prefix = seq.next_element()?.ok_or(A::Error::custom("missing prefix"))?;
                data.player_index = seq.next_element()?.ok_or(A::Error::custom("missing player_index"))?;
                data.turn = seq.next_element()?.ok_or(A::Error::custom("missing turn"))?;
                Ok(data)
            }
        }

        deserializer.deserialize_seq(ChatVisitor)
    }
}

#[derive(Clone, Default)]
pub struct Replay {
    pub version: u32,
    pub id: String,
    pub map_width: u32,
    pub map_height: u32,
    pub usernames: Vec<String>,
    pub stars: Vec<u16>,
    pub cities: Vec<TileIndex>,
    pub city_armies: Vec<u32>,
    pub generals: Vec<TileIndex>,
    pub mountains: Vec<TileIndex>,
    pub moves: Vec<Move>,
    pub afks: Vec<Surrender>,
    pub teams: Option<Vec<u32>>,
    pub map_title: Option<String>,
    pub neutrals: Vec<TileIndex>,
    pub neutral_armies: Vec<TileIndex>,
    pub swamps: Vec<u32>,
    pub chat: Vec<ChatMessage>,
    pub player_colors: Vec<u32>,
    pub lights: Vec<TileIndex>,
    pub options: Options
}