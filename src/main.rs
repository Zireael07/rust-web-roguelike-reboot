extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
use std::cmp::{min, max};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


// This the first roguelike-ish example - a walking @. We build a very simple map,
// and you can use the cursor keys to move around a world.

rltk::add_wasm_support!();
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB };
use rltk::{platform_specific::Command};

//ECS
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}
 
#[derive(Component, Debug)]
struct Player {}

// We'll allow map tiles to be either a wall or a floor. We're deriving PartialEq so we don't
// have to match on it every time. We'll make it a copy type because it's really just an int.
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

// We're extending State to include the ECS world.
struct State {
    ecs: World
}

// We're storing all the tiles in one big array, so we need a way to map an X,Y coordinate to
// a tile. Each row is stored sequentially (so 0..80, 81..160, etc.). This takes an x/y and returns
// the array index.
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

// It's a great idea to have a reverse mapping for these coordinates. This is as simple as
// index % 80 (mod 80), and index / 80
pub fn idx_xy(idx: usize) -> (i32, i32) {
    (idx as i32 % 80, idx as i32 / 80)
}

// Since we have some content, we should also include a map builder. 
fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _ in 0..400 {
        // rand provides a gen_range function to get numbers in a range.
        let x = rng.roll_dice(1, 80) - 1;
        let y = rng.roll_dice(1, 50) - 1;
        let idx = xy_idx(x, y);
        // We don't want to add a wall on top of the player
        if idx != xy_idx(40,25) {
            map[idx] = TileType::Wall;
        }
    }

    // We'll return the state with the short-hand
    map
}

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
fn player_input(gs: &mut State, ctx: &mut Rltk) {
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

fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    // Iterate the map array, incrementing coordinates as we go.
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.print_color(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    ".",
                );
            }
            TileType::Wall => {
                ctx.print_color(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    "#",
                );
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}


impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // Clear the screen
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // Render the player @ symbol
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

//the meat of the EC*S*
impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

// Auto-starts on page load
//start section of the executable may not literally point to main
//#[wasm_bindgen(start)]
//can't use wasm_bindgen(start) because RLTK-rs uses it
pub fn main() {
    
    let context = Rltk::init_simple8x8(80, 50, "RLTK Example 03 - Walking Around", "resources");
    //let gs = State::new();

    //ECS takes more lines to set up
    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    gs.ecs.insert(new_map());

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();

    rltk::main_loop(context, gs);
} 
