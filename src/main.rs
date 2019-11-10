extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
use std::cmp::{min, max};

//better panics
extern crate console_error_panic_hook;
use std::panic;

pub mod camera;
mod gui;
mod gamelog;

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
#[allow(non_snake_case)]
mod NPC_ai_system;
use NPC_ai_system::NPCAI;
mod spawner;
pub mod map_builders;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;

use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB, Point };
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};


// This the code for a roguelike game

rltk::add_wasm_support!();

const SHOW_MAPGEN_VISUALIZER : bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { 
    AwaitingInput, 
    PreRun, 
    PlayerTurn, 
    MonsterTurn,
    MapGeneration
}

// We're extending State to include the ECS world.
pub struct State {
    pub ecs: World,
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

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }
        
        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();                
    
                //paranoia
                if self.mapgen_history.len() > 0 {
                    //draw mapgen
                    camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
    
                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 300.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index == self.mapgen_history.len() {
                            newrunstate = self.mapgen_next_state.unwrap();
                        }
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);

        //draw
        camera::render_camera(&self.ecs, ctx);

        gui::draw_ui(&self.ecs, ctx);

    }
}

//the meat of the EC*S*
impl State {
    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = NPCAI{};
        mob.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl State {
    fn generate_world(&mut self) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
        let mut builder = map_builders::random_builder(&mut rng, 80, 60);
        console::log("Generating world...");
        builder.build_map(&mut rng);

        //prevent borrow checker errors
        std::mem::drop(rng);
        //mapgen visualizer data
        self.mapgen_history = builder.build_data.history.clone();
        //key stuff
        self.ecs.insert(RunState::MapGeneration);
        //self.runstate = RunState::MapGeneration;

        let player_start;
        {
            //fills in map placeholder
            let mut map = self.ecs.write_resource::<Map>();
            *map = builder.build_data.map.clone();
            player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
        }

        //spawn monsters
        builder.spawn_entities(&mut self.ecs);
        //spawn player
        let (player_x, player_y) = (player_start.x, player_start.y);
        let player_entity = spawner::player(&mut self.ecs, player_x, player_y);
        self.ecs.insert(player_entity);
        //special treatment for player location
        self.ecs.insert(Point::new(player_x, player_y));

    }
}

// Auto-starts on page load
//start section of the executable may not literally point to main
//#[wasm_bindgen(start)]
//can't use wasm_bindgen(start) because RLTK-rs uses it
pub fn main() {

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    let mut context = Rltk::init_simple8x8(80, 50, "RLTK Web roguelike", "resources");
    context.with_post_scanlines(true);

    //ECS takes more lines to set up
    let mut gs = State {
        ecs: World::new(),
        //same as actual game starting state
        mapgen_next_state : Some(RunState::PreRun),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Player>();

    //placeholders so that generate_world has stuff to fill
    gs.ecs.insert(Map::new(80,50));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    gs.generate_world();

    gs.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Rusty Roguelike".to_string()] });

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
