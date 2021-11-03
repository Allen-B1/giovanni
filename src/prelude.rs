use std::fmt::Display;

pub use poise::serenity_prelude as serenity;

pub type Data = ();
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
