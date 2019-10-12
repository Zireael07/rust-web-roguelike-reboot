extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys Canvas bindings uses it
use wasm_bindgen::JsCast; // for dyn_into
use std::f64;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[repr(u8)] //single byte representation - enough for our needs since a byte can carry 256 things
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HTMLCommand {
    MoveLeft = 0,
    MoveRight = 1,
    MoveDown = 2,
    MoveUp = 3,
}

#[wasm_bindgen]
pub fn setHTMLCommand(com_id: i32){
        log!("{}", com_id);
        //need to match back to enum
        // https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer
        match com_id {
            com_id if com_id == HTMLCommand::MoveLeft as i32 => log!("{:?}", HTMLCommand::MoveLeft),
            com_id if com_id == HTMLCommand::MoveRight as i32 => log!("{:?}", HTMLCommand::MoveRight),
            com_id if com_id == HTMLCommand::MoveDown as i32 => log!("{:?}", HTMLCommand::MoveDown),
            com_id if com_id == HTMLCommand::MoveUp as i32 => log!("{:?}", HTMLCommand::MoveUp),
            _ => log!("Unknown command"),
}
}

// This the first roguelike-ish example - a walking @. We build a very simple map,
// and you can use the cursor keys to move around a world.

rltk::add_wasm_support!();
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB };

// We'll allow map tiles to be either a wall or a floor. We're deriving PartialEq so we don't
// have to match on it every time. We'll make it a copy type because it's really just an int.
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

// We're extending State to include a minimal map and player coordinates.
struct State {
    map: Vec<TileType>,
    player_position: usize,
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

// Since we have some content, we should also include a map builder. A 'new'
// function is a common Rust way to do this.
impl State {
    pub fn new() -> State {
        let mut state = State {
            map: vec![TileType::Floor; 80 * 50],
            player_position: xy_idx(40, 25),
        };

        // Make the boundaries walls
        for x in 0..80 {
            state.map[xy_idx(x, 0)] = TileType::Wall;
            state.map[xy_idx(x, 49)] = TileType::Wall;
        }
        for y in 0..50 {
            state.map[xy_idx(0, y)] = TileType::Wall;
            state.map[xy_idx(79, y)] = TileType::Wall;
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
            if state.player_position != idx {
                state.map[idx] = TileType::Wall;
            }
        }

        // We'll return the state with the short-hand
        state
    }

    // Handle player movement. Delta X and Y are the relative move
    // requested by the player. We calculate the new coordinates,
    // and if it is a floor - move the player there.
    pub fn move_player(&mut self, delta_x: i32, delta_y: i32) {
        let current_position = idx_xy(self.player_position);
        let new_position = (current_position.0 + delta_x, current_position.1 + delta_y);
        let new_idx = xy_idx(new_position.0, new_position.1);
        if self.map[new_idx] == TileType::Floor {
            self.player_position = new_idx;
        }
    }
}

// Implement the game loop
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // New: handle keyboard inputs.
        match ctx.key {
            None => {} // Nothing happened
            Some(key) => {
                // A key is pressed or held
                match key {
                    // We're matching a key code from GLFW (the GL library underlying RLTK),
                    // and applying movement via the move_player function.

                    // Numpad
                    VirtualKeyCode::Numpad8 => self.move_player(0, -1),
                    VirtualKeyCode::Numpad4 => self.move_player(-1, 0),
                    VirtualKeyCode::Numpad6 => self.move_player(1, 0),
                    VirtualKeyCode::Numpad2 => self.move_player(0, 1),

                    // Numpad diagonals
                    VirtualKeyCode::Numpad7 => self.move_player(-1, -1),
                    VirtualKeyCode::Numpad9 => self.move_player(1, -1),
                    VirtualKeyCode::Numpad1 => self.move_player(-1, 1),
                    VirtualKeyCode::Numpad3 => self.move_player(1, 1),

                    // Cursors
                    VirtualKeyCode::Up => self.move_player(0, -1),
                    VirtualKeyCode::Down => self.move_player(0, 1),
                    VirtualKeyCode::Left => self.move_player(-1, 0),
                    VirtualKeyCode::Right => self.move_player(1, 0),

                    _ => {} // Ignore all the other possibilities
                }
            }
        }

        // Clear the screen
        ctx.cls();

        // Iterate the map array, incrementing coordinates as we go.
        let mut y = 0;
        let mut x = 0;
        for tile in &self.map {
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

        // Render the player @ symbol
        let ppos = idx_xy(self.player_position);
        ctx.print_color(
            ppos.0,
            ppos.1,
            RGB::from_f32(1.0, 1.0, 0.0),
            RGB::from_f32(0., 0., 0.),
            "@",
        );
    }
}



pub fn main_old() {
 let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    log!("We have a context {:?}", context);

    //clear
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("black"));
    context.fill_rect(0.0, 0.0, 800.0, 600.0);
}


// Auto-starts on page load
//start section of the executable may not literally point to main
//#[wasm_bindgen(start)]
//can't use wasm_bindgen(start) because RLTK-rs uses it
pub fn main() {
   //main_old()
    
    let context = Rltk::init_simple8x8(80, 50, "RLTK Example 03 - Walking Around", "resources");
    let gs = State::new();
    rltk::main_loop(context, gs);
} 
