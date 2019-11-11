use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use super::{Position, Player, Viewshed, CombatStats, WantsToMelee, 
    TileType, State, Map, RunState, Entity, Item, WantsToPickupItem,
    gamelog::GameLog};
use std::cmp::{min, max};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

// Handle player movement. Delta X and Y are the relative move
// requested by the player. We calculate the new coordinates,
// and if it is a floor - move the player there.
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    // let current_position = idx_xy(self.player_position);
    // let new_position = (current_position.0 + delta_x, current_position.1 + delta_y);
    // let new_idx = xy_idx(new_position.0, new_position.1);
    // if self.map[new_idx] == TileType::Floor {
    //     self.player_position = new_idx;
    // }
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        //paranoia
        if (pos.x + delta_x) > 0 && (pos.y + delta_y) > 0 {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            if destination_idx > 0 && destination_idx < map.tiles.len() {
                //handle attacking
                for potential_target in map.tile_content[destination_idx].iter() {
                    let target = combat_stats.get(*potential_target);
                    if let Some(_target) = target {
                        wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed");
                        //console::log(&format!("We want to melee: {:?}", target));
                        return;
                    }
                }


                if !map.blocked[destination_idx] {
                    pos.x = min(map.width-1, max(0, pos.x + delta_x));
                    pos.y = min(map.height-1, max(0, pos.y + delta_y));

                    //mark our FoV as dirty after a move
                    viewshed.dirty = true;

                    //update player location data
                    let mut ppos = ecs.write_resource::<Point>();
                    ppos.x = pos.x;
                    ppos.y = pos.y;
                }
            }
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();    

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        // the tutorial inserts at 0, so the latest is at the top. we do what is more usual, append, so the latest is at bottom
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
}

// Implement the game loop
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    // New: Handle web buttons
    if let Some(btn) = &ctx.web_button {
        match btn.trim() {
            "go_nw" => try_move_player(-1, -1, &mut gs.ecs),
            "go_n" => try_move_player(0, -1, &mut gs.ecs),
            "go_ne" => try_move_player(1, -1, &mut gs.ecs),
            "go_w" => try_move_player(-1, 0, &mut gs.ecs),
            "go_e" => try_move_player(1, 0, &mut gs.ecs),
            "go_sw" => try_move_player(-1, 1, &mut gs.ecs),
            "go_s" => try_move_player(0, 1, &mut gs.ecs),
            "go_se" => try_move_player(1, 1, &mut gs.ecs),
            //skip turn
            "go_wait" => return RunState::PlayerTurn,
            _ => { return RunState::AwaitingInput } //Nothing happened
        }
    }
    else {
        // New: handle keyboard inputs.
        match ctx.key {
            None => { return RunState::AwaitingInput } // Nothing happened
            Some(key) => {
                // A key is pressed or held
                match key {
                    // We're matching a key code from GLFW (the GL library underlying RLTK),
                    // and applying movement via the move_player function.

                    // Numpad
                    VirtualKeyCode::Numpad8 => try_move_player(0, -1, &mut gs.ecs),
                    VirtualKeyCode::Numpad4 => try_move_player(-1, 0, &mut gs.ecs),
                    VirtualKeyCode::Numpad6 => try_move_player(1, 0, &mut gs.ecs),
                    VirtualKeyCode::Numpad2 => try_move_player(0, 1, &mut gs.ecs),

                    // Numpad diagonals
                    VirtualKeyCode::Numpad7 => try_move_player(-1, -1, &mut gs.ecs),
                    VirtualKeyCode::Numpad9 => try_move_player(1, -1, &mut gs.ecs),
                    VirtualKeyCode::Numpad1 => try_move_player(-1, 1, &mut gs.ecs),
                    VirtualKeyCode::Numpad3 => try_move_player(1, 1, &mut gs.ecs),

                    //vi keys
                    VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
                    VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
                    VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
                    VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

                    // Cursors
                    VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
                    VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
                    VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
                    VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),

                    // Skip turn
                    VirtualKeyCode::Numpad5 => return RunState::PlayerTurn,
                    VirtualKeyCode::Space => return RunState::PlayerTurn,

                    VirtualKeyCode::G => get_item(&mut gs.ecs),
                    VirtualKeyCode::I => return RunState::ShowInventory,
                    VirtualKeyCode::D => return RunState::ShowDropItem,


                    _ => { return RunState::AwaitingInput } // Nothing happened, ignore all the other possibilities
                }
            }
        }
    }

    RunState::PlayerTurn
}
