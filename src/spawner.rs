extern crate rltk;
use rltk::{ RGB, RandomNumberGenerator };
extern crate specs;
use specs::prelude::*;
use super::{Player, Renderable, Name, Position, Viewshed, Monster, Rect, Map, TileType,
BlocksTile, CombatStats, Item, MedItem, Consumable, Ranged, InflictsDamage, AreaOfEffect, Confusion, 
Equippable, EquipmentSlot, MeleePowerBonus, DefenseBonus, Attributes, Attribute, Pools, Pool,
random_table::RandomTable, raws::*};
use crate::{attr_bonus};
use std::collections::HashMap; //for region spawning
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range : 8, dirty: true })
        //the player absolutely needs this as without it, combat doesn't work
        .with(Name{ name : "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .with(Pools{
            hit_points : Pool{ 
                current: 30, 
                max: 30 
            },
        })
        .with(Attributes{
            strength: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            dexterity: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            constitution: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            wisdom: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            charisma: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11)},
        })
        .with(Player{})
        .build()
}


fn room_table() -> RandomTable {
    //the spawn table has been JSONized
    get_spawn_table(&RAWS.lock().unwrap())
}

pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room : &Rect, list_spawns : &mut Vec<(usize, String)>) {
    let mut possible_targets : Vec<usize> = Vec::new();
    { // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, list_spawns);
}

pub fn spawn_region(map: &Map, rng: &mut RandomNumberGenerator, area : &[usize], list_spawns : &mut Vec<(usize, String)>) {
    let mut spawn_points : HashMap<usize, String> = HashMap::new();
    let mut areas : Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let num_spawns = 1;
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            //paranoia
            if areas.len() as i32-1 < 0 {
                console::log(&format!("Roll {:?} ", areas.len() as i32-1));
                return
            }
            
            let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };
            let map_idx = areas[array_index];
            
            spawn_points.insert(map_idx, room_table().roll(rng));
            areas.remove(array_index);
        }

        // //Spawn an item per room
        // //paranoia
        // if areas.len() > 0 {
        //     let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };
        //     let map_idx = areas[array_index];
        //     spawn_points.insert(map_idx, random_select_item_roll(rng));
        //     areas.remove(array_index);
        // }
    }




    // Prepare to spawn
    for spawn in spawn_points.iter() {
        list_spawns.push((*spawn.0, spawn.1.to_string()));
        //spawn_entity(ecs, &spawn);
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, spawn : &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    //spawn from data
    let spawn_result = spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), &spawn.1, SpawnType::AtPosition{ x, y});
    if spawn_result.is_some() {
        return;
    }

    console::log(&format!("WARNING: We don't know how to spawn [{}]!", spawn.1));
}

///Random selection
pub fn random_select_roll(rng: &mut RandomNumberGenerator) -> String {
    let roll :i32;
    {
        //random selection
        //let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => "Human".to_string(),
        _ => "Cop".to_string(),
    }
}


pub fn random_select_item_roll(rng: &mut RandomNumberGenerator) -> String {
    let roll :i32;
    {
        //random selection
        roll = rng.roll_dice(1, 4);
    }
    match roll {
        1 => "Medkit".to_string(),
        2 => "Grenade".to_string(),
        3 => "Concussion Grenade".to_string(),
        _ => "Pistol".to_string(),
    }
}

//items and monsters are now spawned from JSON data