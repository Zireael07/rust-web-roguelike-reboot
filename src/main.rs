extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
use std::cmp::{min, max};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod NPC_ai_system;
use NPC_ai_system::NPCAI;
mod spawner;
pub mod map_builders;

use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB, Point };

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


// This the code for a roguelike game

rltk::add_wasm_support!();

const SHOW_MAPGEN_VISUALIZER : bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, 
    Running,
    MapGeneration
}

// We're extending State to include the ECS world.
pub struct State {
    pub ecs: World,
    //necessary for turn-basedness
    pub runstate : RunState,
    //mapgen visualizer stuff that has nowhere else to go
    mapgen_next_state : Option<RunState>,
    mapgen_history : Vec<Map>,
    mapgen_index : usize,
    mapgen_timer : f32
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // Clear the screen
        ctx.cls();

        //mapgen visualization
        if self.runstate == RunState::MapGeneration {
            if !SHOW_MAPGEN_VISUALIZER {
                self.runstate = self.mapgen_next_state.unwrap();
            }
            ctx.cls();                
            draw_map(&self.mapgen_history[self.mapgen_index], ctx);

            self.mapgen_timer += ctx.frame_time_ms;
            if self.mapgen_timer > 300.0 {
                self.mapgen_timer = 0.0;
                self.mapgen_index += 1;
                if self.mapgen_index == self.mapgen_history.len() {
                    self.runstate = self.mapgen_next_state.unwrap();
                }
            }
        }

        //turn-basedness
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else if self.runstate != RunState::MapGeneration {
            self.runstate = player_input(self, ctx);
        }

        //normal map drawing
        let map = self.ecs.fetch::<Map>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // Render the player @ symbol
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph); }
        }
    }
}

//the meat of the EC*S*
impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = NPCAI{};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

// Auto-starts on page load
//start section of the executable may not literally point to main
//#[wasm_bindgen(start)]
//can't use wasm_bindgen(start) because RLTK-rs uses it
pub fn main() {
    
    let context = Rltk::init_simple8x8(80, 50, "RLTK Web roguelike", "resources");
    //let gs = State::new();

    //ECS takes more lines to set up
    let mut gs = State {
        ecs: World::new(),
        runstate : RunState::Running,
        //same as actual game starting state
        mapgen_next_state : Some(RunState::Running),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();

    //the builder object is now kept
    let mut builder = map_builders::random_builder();
    builder.build_map();
    let mut map = builder.get_map();
    //mapgen visualizer data
    gs.mapgen_history = builder.get_snapshot_history();

    gs.runstate = RunState::MapGeneration;

    gs.ecs.insert(map);

    let start = builder.get_starting_position();
    let (player_x, player_y) = (start.x, start.y);

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    //spawn monsters
    builder.spawn_entities(&mut gs.ecs);

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    //special treatment for player location
    gs.ecs.insert(Point::new(player_x, player_y));

    //register html buttons
    rltk::register_html_button("go_nw");
    rltk::register_html_button("go_n");
    rltk::register_html_button("go_ne");
    rltk::register_html_button("go_w");
    rltk::register_html_button("go_e");
    rltk::register_html_button("go_sw");
    rltk::register_html_button("go_s");
    rltk::register_html_button("go_se");

    rltk::main_loop(context, gs);
} 
