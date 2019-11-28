use std::collections::{HashMap, HashSet};
use specs::prelude::*;
use crate::components::*;
use super::{Raws};
use crate::random_table::{RandomTable};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};
use crate::{attr_bonus};
use regex::Regex;

pub fn parse_dice_string(dice : &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE : Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }

    }
    (n_dice, die_type, die_bonus)
}


pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity }
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
pub fn string_to_slot(slot : &str) -> EquipmentSlot {
    match slot {
        "Shield" => EquipmentSlot::Shield, 
        "Head" => EquipmentSlot::Head,
        "Torso" => EquipmentSlot::Torso, 
        "Legs" => EquipmentSlot::Legs, 
        "Feet" => EquipmentSlot::Feet, 
        "Hands" => EquipmentSlot::Hands,
        "Melee" => EquipmentSlot::Melee,
        _ => { println!("Warning: unknown equipment slot type [{}])", slot); EquipmentSlot::Melee }
    }
}

fn find_slot_for_equippable_item(tag : &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(_wpn) = &item.weapon {
        return EquipmentSlot::Melee;
    } else if let Some(wearable) = &item.wearable {
        return string_to_slot(&wearable.slot);
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

//lifetime marker <'a> necessary because we're accessing raws, which we need to find item's slot
fn spawn_position<'a>(pos : SpawnType, new_entity : EntityBuilder<'a>, tag : &str, raws: &RawMaster) -> EntityBuilder<'a> {
    let eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition{x,y} => eb.with(Position{ x, y }),
        SpawnType::Carried{by} => eb.with(InBackpack{ owner: by }),
        SpawnType::Equipped{by} => {
            let slot = find_slot_for_equippable_item(tag, raws);
            eb.with(Equipped{ owner: by, slot })
        }
    }

}

fn get_renderable_component(renderable : &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable{  
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg : rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg : rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order : renderable.order,
    }
}

//we can't pass in both the ECS and the entity because the entity keeps hold of a reference to ECS
pub fn spawn_named_entity(raws: &RawMaster, ecs : &mut World, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }

    None
}

pub fn spawn_named_item(raws: &RawMaster, ecs: &mut World, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut eb = ecs.create_entity();
        eb = spawn_position(pos, eb, key, raws);

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
            let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
            let mut wpn = MeleeWeapon{
                damage_n_dice : n_dice,
                damage_die_type : die_type,
                damage_bonus : bonus,
            };
            eb = eb.with(wpn);
        }
        
        if let Some(wearable) = &item_template.wearable {
            let slot = string_to_slot(&wearable.slot);
            eb = eb.with(Equippable{ slot: slot });
            eb = eb.with(DefenseBonus{ defense: wearable.defense_bonus });
        }

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(raws: &RawMaster, ecs: &mut World, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let mut eb = ecs.create_entity();

        // Spawn in the specified location
        eb = spawn_position(pos, eb, key, raws);

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

        let pools = Pools{
            hit_points : Pool{ current: mob_template.stats.hp, max: mob_template.stats.max_hp },
        };
        eb = eb.with(pools);

        //handle attributes (default of 11 unless specified)
        let mut attr = Attributes{
            strength: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            dexterity: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            constitution: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            wisdom: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            charisma: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) }
        };
        if let Some(strength) = mob_template.attributes.strength { 
            attr.strength = Attribute{ base: strength, modifiers: 0, bonus: attr_bonus(strength) }; 
        }
        if let Some(dexterity) = mob_template.attributes.dexterity { 
            attr.dexterity = Attribute{ base: dexterity, modifiers: 0, bonus: attr_bonus(dexterity) }; 
        }
        if let Some(constitution) = mob_template.attributes.constitution { 
            attr.constitution = Attribute{ base: constitution, modifiers: 0, bonus: attr_bonus(constitution) }; 
        }
        if let Some(intelligence) = mob_template.attributes.intelligence { 
            attr.intelligence = Attribute{ base: intelligence, modifiers: 0, bonus: attr_bonus(intelligence) }; 
        }
        if let Some(wisdom) = mob_template.attributes.wisdom { 
            attr.wisdom = Attribute{ base: wisdom, modifiers: 0, bonus: attr_bonus(wisdom) }; 
        }
        if let Some(charisma) = mob_template.attributes.charisma { 
            attr.charisma = Attribute{ base: charisma, modifiers: 0, bonus: attr_bonus(charisma) }; 
        }

        eb = eb.with(attr);

        eb = eb.with(Viewshed{ visible_tiles : Vec::new(), range: mob_template.vision_range, dirty: true });

        // Initiative of 2
        eb = eb.with(Initiative{current: 2});

        let new_mob = eb.build();

        // Are they wielding anyting?
        if let Some(wielding) = &mob_template.equipped {
            for tag in wielding.iter() {
                spawn_named_entity(raws, ecs, tag, SpawnType::Equipped{ by: new_mob });
            }
        }

        return Some(new_mob);
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, ecs: &mut World, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut eb = ecs.create_entity();

        // Spawn in the specified location
        eb = spawn_position(pos, eb, key, raws);

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