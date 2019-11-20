use serde::{Deserialize};
use super::{Renderable};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Prop {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub hidden : Option<bool>,
    pub entry_trigger : Option<EntryTrigger>
}

#[derive(Deserialize, Debug)]
pub struct EntryTrigger {
    pub effects : HashMap<String, String>
}