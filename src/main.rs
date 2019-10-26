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

use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB, Point };

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


// This the code for a roguelike game

rltk::add_wasm_support!();

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

// We're extending State to include the ECS world.
pub struct State {
    pub ecs: World,
    //necessary for turn-basedness
    pub runstate : RunState
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // Clear the screen
        ctx.cls();

        //turn-basedness
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

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
        runstate : RunState::Running
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    //spawn monsters
    //we skip room 1 because we don't want any in starting room
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();

        spawner::random_monster(&mut gs.ecs, x, y);
    }

    gs.ecs.insert(map);

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
