use serde::{Deserialize};
use super::{Renderable};

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub blocks_tile : bool,
    pub stats : MobStats,
    pub vision_range : i32,
    pub movement : String,
    pub quips : Option<Vec<String>>,
    pub attributes : MobAttributes,
    pub equipped : Option<Vec<String>>,
    pub faction : Option<String>,
    pub money : Option<String>, //dice roll
    pub vendor : Option<Vec<String>>
}

#[derive(Deserialize, Debug)]
pub struct MobStats {
    pub max_hp : i32,
    pub hp : i32,
    pub power : i32,
    pub defense : i32
}

#[derive(Deserialize, Debug)]
pub struct MobAttributes {
    pub strength : Option<i32>,
    pub dexterity : Option<i32>,
    pub constitution : Option<i32>,
    pub intelligence : Option<i32>,
    pub wisdom : Option<i32>,
    pub charisma : Option<i32>
}