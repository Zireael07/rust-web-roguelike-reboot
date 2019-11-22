use serde::{Deserialize};
use super::{Renderable};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Prop {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub hidden : Option<bool>,
    pub entry_trigger : Option<EntryTrigger>,
    pub blocks_tile : Option<bool>,
    pub blocks_visibility : Option<bool>,
    pub door_open : Option<bool>,
    pub light : Option<Light>
}

#[derive(Deserialize, Debug)]
pub struct EntryTrigger {
    pub effects : HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct Light {
    pub range : i32,
    pub color : String
}