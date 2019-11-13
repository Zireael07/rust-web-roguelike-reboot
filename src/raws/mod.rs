mod item_structs;
use item_structs::*;
use serde::{Deserialize};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

rltk::embedded_resource!(RAW_FILE, "../../data/spawns.json");

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items : Vec<Item>
}

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../data/spawns.json");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = rltk::embedding::EMBED
    .lock()
    .unwrap()
    .get_resource("../../data/spawns.json".to_string())
    .unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let decoder : Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");
    console::log(&format!("{:?}", decoder));
}