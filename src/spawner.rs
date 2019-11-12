extern crate rltk;
use rltk::{ RGB, RandomNumberGenerator };
extern crate specs;
use specs::prelude::*;
use super::{Player, Renderable, Name, Position, Viewshed, Monster, Rect, Map, TileType,
BlocksTile, CombatStats, Item, MedItem, Consumable, Ranged, InflictsDamage, AreaOfEffect, Confusion,
random_table::RandomTable};
use std::collections::HashMap; //for region spawning

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range : 8, dirty: true })
        //the player absolutely needs this as without it, combat doesn't work
        .with(Name{ name : "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .with(Player{})
        .build()
}


fn room_table() -> RandomTable {
    RandomTable::new()
        .add("Human", 10)
        .add("Cop", 5)
        .add("Medkit", 7)
        .add("Pistol", 4)
        .add("Concussion Grenade", 2)
        .add("Grenade", 3)
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

    match spawn.1.as_ref() {
        "Human" => human(ecs, x, y),
        "Cop" => cop(ecs, x, y),
        "Medkit" => medkit(ecs, x, y),
        "Pistol" => pistol(ecs, x, y),
        "Grenade" => grenade(ecs, x, y),
        "Concussion Grenade" => concussion_grenade(ecs, x, y),
        _ => {}
    }
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

/// Spawns a random monster at a given location
#[allow(dead_code)]
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        //random selection
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { human(ecs, x, y) }
        _ => { cop(ecs, x, y) }
    }
}

fn human(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('h'), "Human"); }
fn cop(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('c'), "Cop"); }

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : u8, name : S) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .build();
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

#[allow(dead_code)]
fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 4);
    }
    match roll {
        1 => { medkit(ecs, x, y) }
        2 => { grenade(ecs,x, y) }
        3 => { concussion_grenade(ecs, x, y) }
        _ => { pistol(ecs, x, y) }
    }
}

fn medkit(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('!'), //looks like an injector
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Medkit".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(MedItem{ heal_amount: 8 })
        .build();
}

//refluffed magic missile scroll :P
fn pistol(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Pistol".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 8 })
        .build();
}

//refluffed fireball
fn grenade(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('*'), //pity we can't use Unicode yet
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Grenade".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 20 })
        .with(AreaOfEffect{ radius: 3 })
        .build();
}

//refluffed confusion scroll
fn concussion_grenade(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('*'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Concussion Grenade".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(Confusion{ turns: 4 })
        .build();
}