use rltk::{VirtualKeyCode, Rltk};
use specs::prelude::*;
use super::{Position, Player, TileType, xy_idx, State};
use std::cmp::{min, max};

use rltk::{platform_specific::Command};

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
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79 , max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

// Implement the game loop
pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    // HTML input
    match ctx.command {
        None => {} //Nothing
        Some(command) => {
            match command {
                Command::MoveLeft => try_move_player(-1, 0, &mut gs.ecs),
                Command::MoveRight => try_move_player(1, 0, &mut gs.ecs),
                Command::MoveUp => try_move_player(0,-1, &mut gs.ecs),
                Command::MoveDown => try_move_player(0,1, &mut gs.ecs),
                    _ => {} // Ignore all the other possibilities
            }
        }
    }
        
    // New: handle keyboard inputs.
    match ctx.key {
        None => {} // Nothing happened
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

                _ => {} // Ignore all the other possibilities
            }
        }
    }
}
