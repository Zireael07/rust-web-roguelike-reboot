use serde::{Deserialize};
use super::{Renderable};

#[derive(Deserialize, Debug)]
pub struct SpawnTableEntry {
    pub name : String,
    pub weight : i32
}