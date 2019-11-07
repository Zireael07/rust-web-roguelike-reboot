use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use super::{Position, Player, Viewshed, TileType, State, Map, RunState};
use std::cmp::{min, max};

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
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        //paranoia
        if (pos.x + delta_x) > 0 && (pos.y + delta_y) > 0 {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            if destination_idx > 0 && destination_idx < map.tiles.len() {
                if map.tiles[destination_idx] != TileType::Wall {
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
            _ => { return RunState::Paused } //Nothing happened
        }
    }
    else {
        // New: handle keyboard inputs.
        match ctx.key {
            None => { return RunState::Paused } // Nothing happened
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

                    // Cursors
                    VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
                    VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
                    VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
                    VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),

                    _ => { return RunState::Paused } // Nothing happened, ignore all the other possibilities
                }
            }
        }
    }

    RunState::Running
}
