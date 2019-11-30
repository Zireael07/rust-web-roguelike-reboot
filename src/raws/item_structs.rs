use serde::{Deserialize};
use std::collections::HashMap;


#[derive(Deserialize, Debug)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>,
    pub weapon : Option<Weapon>,
    pub wearable : Option<Wearable>,
    pub weight_lbs : Option<f32>,
    pub base_value : Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg : String,
    pub bg : String,
    pub order: i32
}

#[derive(Deserialize, Debug)]
pub struct Consumable {
    pub effects : HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct Weapon {
    pub range: String,
    pub base_damage: String //because it's a dice string
}

#[derive(Deserialize, Debug)]
pub struct Wearable {
    pub defense_bonus: f32,
    pub slot : String
}