use serde::{Deserialize,Serialize};

#[derive(Serialize, Deserialize)]
pub struct Player {
    name: String,
    currentName: String,
    // hopefully they implement fractional stars!!!!
    stars: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub started: u64,
    pub turns: u64,
    pub ranking: Vec<Player>
}