use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use super::{Position, Player, Viewshed, Pools, WantsToMelee, 
    TileType, State, Map, RunState, Entity, Item, WantsToPickupItem, EntityMoved, 
    Faction, raws::Reaction, Vendor, VendorMode,
    Door, BlocksVisibility, BlocksTile, Renderable, gamelog::GameLog};
use std::cmp::{min, max};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

// Handle player movement. 
// Delta X and Y are the relative move
// requested by the player. We calculate the new coordinates,
// and if it is a floor - move the player there.
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let pools = ecs.read_storage::<Pools>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let map = ecs.fetch::<Map>();
    //doors
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    //non-hostile NPCs
    let factions = ecs.read_storage::<Faction>();
    let vendors = ecs.read_storage::<Vendor>();
    let mut result = RunState::AwaitingInput;

    let mut swap_entities : Vec<(Entity, i32, i32)> = Vec::new();

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        //paranoia
        if (pos.x + delta_x) > 0 && (pos.y + delta_y) > 0 {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            if destination_idx > 0 && destination_idx < map.tiles.len() {

                //handle move targets
                for potential_target in map.tile_content[destination_idx].iter() {
                    //handle vendors first
                    if let Some(_vendor) = vendors.get(*potential_target) {
                        return RunState::ShowVendor{ vendor: *potential_target, mode : VendorMode::Sell }
                    }
                    let mut hostile = true;
                    if pools.get(*potential_target).is_some() {
                        // is it hostile?
                        if let Some(faction) = factions.get(*potential_target) {
                            let reaction = crate::raws::faction_reaction(
                                &faction.name, 
                                "Player", 
                                &crate::raws::RAWS.lock().unwrap()
                            );
                            console::log(format!("Reaction: {:?} ", reaction));
                            if reaction != Reaction::Attack { hostile = false; }
                        }
                    }
                    if !hostile {
                        // Note that we want to move the bystander
                        swap_entities.push((*potential_target, pos.x, pos.y));

                        // Move the player
                        pos.x = min(map.width-1 , max(0, pos.x + delta_x));
                        pos.y = min(map.height-1, max(0, pos.y + delta_y));
                        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");

                        viewshed.dirty = true;
                        let mut ppos = ecs.write_resource::<Point>();
                        ppos.x = pos.x;
                        ppos.y = pos.y;
                    } else {
                        //handle attacking
                        let target = pools.get(*potential_target);
                        if let Some(_target) = target {
                            wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed");
                            //console::log(&format!("We want to melee: {:?}", target));
                            return RunState::Ticking;
                        }
                    }
                    let door = doors.get_mut(*potential_target);
                    if let Some(door) = door {
                        //unblock visibility, movement and change the glyph
                        door.open = true;
                        blocks_visibility.remove(*potential_target);
                        blocks_movement.remove(*potential_target);
                        let glyph = renderables.get_mut(*potential_target).unwrap();
                        glyph.glyph = rltk::to_cp437('±');
                        viewshed.dirty = true;
                        result = RunState::Ticking;
                    }
                }


                if !map.blocked[destination_idx] {
                    pos.x = min(map.width-1, max(0, pos.x + delta_x));
                    pos.y = min(map.height-1, max(0, pos.y + delta_y));

                    entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                    
                    //mark our FoV as dirty after a move
                    viewshed.dirty = true;

                    //update player location data
                    let mut ppos = ecs.write_resource::<Point>();
                    ppos.x = pos.x;
                    ppos.y = pos.y;
                    result = RunState::Ticking;
                }
            }
        }
    } //for loop ends here
    
    //new positions for swapped entities
    for m in swap_entities.iter() {
        let their_pos = positions.get_mut(m.0);
        if let Some(their_pos) = their_pos {
            their_pos.x = m.1;
            their_pos.y = m.2;
        }
    }

    result
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
            "go_nw" => return try_move_player(-1, -1, &mut gs.ecs),
            "go_n" => return try_move_player(0, -1, &mut gs.ecs),
            "go_ne" => return try_move_player(1, -1, &mut gs.ecs),
            "go_w" => return try_move_player(-1, 0, &mut gs.ecs),
            "go_e" => return try_move_player(1, 0, &mut gs.ecs),
            "go_sw" => return try_move_player(-1, 1, &mut gs.ecs),
            "go_s" => return try_move_player(0, 1, &mut gs.ecs),
            "go_se" => return try_move_player(1, 1, &mut gs.ecs),
            //skip turn
            "go_wait" => return RunState::Ticking,
            //others
            "get" => get_item(&mut gs.ecs),
            "inven" => return RunState::ShowInventory,
            "drop" => return RunState::ShowDropItem,
            "remove" => return RunState::ShowRemoveItem,
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
                    VirtualKeyCode::Numpad8 => return try_move_player(0, -1, &mut gs.ecs),
                    VirtualKeyCode::Numpad4 => return try_move_player(-1, 0, &mut gs.ecs),
                    VirtualKeyCode::Numpad6 => return try_move_player(1, 0, &mut gs.ecs),
                    VirtualKeyCode::Numpad2 => return try_move_player(0, 1, &mut gs.ecs),

                    // Numpad diagonals
                    VirtualKeyCode::Numpad7 => return try_move_player(-1, -1, &mut gs.ecs),
                    VirtualKeyCode::Numpad9 => return try_move_player(1, -1, &mut gs.ecs),
                    VirtualKeyCode::Numpad1 => return try_move_player(-1, 1, &mut gs.ecs),
                    VirtualKeyCode::Numpad3 => return try_move_player(1, 1, &mut gs.ecs),

                    //vi keys
                    VirtualKeyCode::Y => return try_move_player(-1, -1, &mut gs.ecs),
                    VirtualKeyCode::U => return try_move_player(1, -1, &mut gs.ecs),
                    VirtualKeyCode::B => return try_move_player(-1, 1, &mut gs.ecs),
                    VirtualKeyCode::N => return try_move_player(1, 1, &mut gs.ecs),

                    // Cursors
                    VirtualKeyCode::Up => return try_move_player(0, -1, &mut gs.ecs),
                    VirtualKeyCode::Down => return try_move_player(0, 1, &mut gs.ecs),
                    VirtualKeyCode::Left => return try_move_player(-1, 0, &mut gs.ecs),
                    VirtualKeyCode::Right => return try_move_player(1, 0, &mut gs.ecs),

                    // Skip turn
                    VirtualKeyCode::Numpad5 => return RunState::Ticking,
                    VirtualKeyCode::Space => return RunState::Ticking,

                    VirtualKeyCode::G => get_item(&mut gs.ecs),
                    VirtualKeyCode::I => return RunState::ShowInventory,
                    VirtualKeyCode::D => return RunState::ShowDropItem,
                    VirtualKeyCode::R => return RunState::ShowRemoveItem,


                    _ => { return RunState::AwaitingInput } // Nothing happened, ignore all the other possibilities
                }
            }
        }
    }

    RunState::Ticking
}
