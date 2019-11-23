use std::collections::{HashMap, HashSet};
use specs::prelude::*;
use crate::components::*;
use super::{Raws};
use crate::random_table::{RandomTable};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 }
}

pub struct RawMaster {
    raws : Raws,
    item_index : HashMap<String, usize>,
    mob_index : HashMap<String, usize>,
    prop_index : HashMap<String, usize>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{ items: Vec::new(), mobs: Vec::new(), props: Vec::new(), spawn_table: Vec::new() },
            item_index : HashMap::new(),
            mob_index : HashMap::new(),
            prop_index : HashMap::new()
        }
    }

    pub fn load(&mut self, raws : Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        let mut used_names : HashSet<String> = HashSet::new();
        for (i,item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                console::log(&format!("WARNING -  duplicate item name in raws [{}]", item.name));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i,mob) in self.raws.mobs.iter().enumerate() {
            self.mob_index.insert(mob.name.clone(), i);
            if used_names.contains(&mob.name) {
                console::log(&format!("WARNING -  duplicate mob name in raws [{}]", mob.name));
            }
            used_names.insert(mob.name.clone());
        }
        for (i,prop) in self.raws.props.iter().enumerate() {
            self.prop_index.insert(prop.name.clone(), i);
            if used_names.contains(&prop.name) {
                console::log(&format!("WARNING -  duplicate mob name in raws [{}]", prop.name));
            }
            used_names.insert(prop.name.clone());
        }
        // print error if doesn't exist
        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                console::log(&format!("WARNING - Spawn tables references unspecified entity {}", spawn.name));
            }
        }
    }
    
}

//have to be outside RawMaster impl

pub fn get_spawn_table(raws: &RawMaster) -> RandomTable {
    use super::SpawnTableEntry;

    let available_options : Vec<&SpawnTableEntry> = raws.raws.spawn_table
        .iter()
    //    .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();
    
    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        rt = rt.add(e.name.clone(), weight);
    }

    rt
}

//helpers
fn spawn_position(pos : SpawnType, new_entity : EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition{x,y} => {
            eb = eb.with(Position{ x, y });
        }
    }

    eb
}

fn get_renderable_component(renderable : &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable{  
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg : rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg : rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order : renderable.order,
    }
}

pub fn spawn_named_entity(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, new_entity, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, new_entity, key, pos);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, new_entity, key, pos);
    }

    None
}

pub fn spawn_named_item(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut eb = new_entity;
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
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

        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable{ slot: EquipmentSlot::Melee });
            eb = eb.with(MeleePowerBonus{ power : weapon.power_bonus });
        }
        
        if let Some(shield) = &item_template.shield {
            eb = eb.with(Equippable{ slot: EquipmentSlot::Shield });
            eb = eb.with(DefenseBonus{ defense: shield.defense_bonus });
        }

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name{ name : mob_template.name.clone() });

        match mob_template.ai.as_ref() {
            "melee" => eb = eb.with(Monster{}),
            "bystander" => eb = eb.with(Bystander{}),
            "vendor" => eb = eb.with(Vendor{}),
            _ => {}
        }

        if let Some(quips) = &mob_template.quips {
            eb = eb.with(Quips{
                available: quips.clone()
            });
        }
        
        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile{});
        }
        eb = eb.with(CombatStats{
            max_hp : mob_template.stats.max_hp,
            hp : mob_template.stats.hp,
            power : mob_template.stats.power,
            defense : mob_template.stats.defense
        });
        eb = eb.with(Viewshed{ visible_tiles : Vec::new(), range: mob_template.vision_range, dirty: true });

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name{ name : prop_template.name.clone() });

        if let Some(hidden) = prop_template.hidden {
            if hidden { eb = eb.with(Hidden{}) };
        }
        if let Some(entry_trigger) = &prop_template.entry_trigger {
            eb = eb.with(EntryTrigger{});
            for effect in entry_trigger.effects.iter() {
                match effect.0.as_str() {
                    "damage" => { eb = eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }) }
                    "single_activation" => { eb = eb.with(SingleActivation{}) }
                    _ => {}
                }
            }
        }
        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile { eb = eb.with(BlocksTile{}) };
        }
        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility { eb = eb.with(BlocksVisibility{}) };
        }
        if let Some(door_open) = prop_template.door_open {
            eb = eb.with(Door{ open: door_open });
        }
        if let Some(light) = &prop_template.light {
            eb = eb.with(LightSource{ range: light.range, color : rltk::RGB::from_hex(&light.color).expect("Bad color") });
            eb = eb.with(Viewshed{ range: light.range, dirty: true, visible_tiles: Vec::new() });
        }

        return Some(eb.build());
    }
    None
}