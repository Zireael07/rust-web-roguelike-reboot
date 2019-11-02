extern crate rltk;
use rltk::{ RGB, RandomNumberGenerator };
extern crate specs;
use specs::prelude::*;
use super::{Player, Renderable, Name, Position, Viewshed, Monster, 
Rect, Map, TileType};
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
        .with(Player{})
        .build()
}

pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let mut possible_targets : Vec<usize> = Vec::new();
    { // Borrow scope - to keep access to the map separated
        let map = ecs.fetch::<Map>();
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(ecs, &possible_targets);
}

pub fn spawn_region(ecs: &mut World, area : &[usize]) {
    let mut spawn_points : HashMap<usize, String> = HashMap::new();
    let mut areas : Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = 1;
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, random_select_roll(&mut rng));
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_entity(ecs, &spawn);
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
fn spawn_entity(ecs: &mut World, spawn : &(&usize, &String)) {
    let x = (*spawn.0 % 80) as i32;
    let y = (*spawn.0 / 80) as i32;

    match spawn.1.as_ref() {
        "Human" => human(ecs, x, y),
        "Cop" => cop(ecs, x, y),
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
        .build();
}