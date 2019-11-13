use std::collections::HashMap;
use specs::prelude::*;
use crate::components::*;
use super::{Raws};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 }
}

pub struct RawMaster {
    raws : Raws,
    item_index : HashMap<String, usize>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{ items: Vec::new() },
            item_index : HashMap::new()
        }
    }

    pub fn load(&mut self, raws : Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i,item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
    }
    
}

//has to be outside RawMaster impl
pub fn spawn_named_item(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        match pos {
            SpawnType::AtPosition{x,y} => {
                eb = eb.with(Position{ x, y });
            }
        }

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(crate::components::Renderable{  
                glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
                fg : rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
                bg : rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
            });
        }

        eb = eb.with(Name{ name : item_template.name.clone() });

        eb = eb.with(crate::components::Item{});

        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(crate::components::Consumable{});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "med_item" => { 
                        eb = eb.with(MedItem{ heal_amount: effect.1.parse::<i32>().unwrap() }) 
                    }
                    "ranged" => { eb = eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }) },
                    "damage" => { eb = eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }) }
                    "area_of_effect" => { eb = eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }) }
                    "confusion" => { eb = eb.with(Confusion{ turns: effect.1.parse::<i32>().unwrap() }) }
                    _ => {
                        println!("Warning: consumable effect {} not implemented.", effect_name);
                    }
                }
            }
        }

        return Some(eb.build());
    }
    None
}