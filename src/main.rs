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
mod ai;
mod spawner;
pub mod map_builders;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod inventory_system;
use inventory_system::*;
mod trigger_system;
use trigger_system::TriggerSystem;
pub mod random_table;
pub mod particle_system;
pub mod lighting_system;
mod gamesystem;
pub use gamesystem::*;

//load json data
pub mod raws;
#[macro_use]
extern crate lazy_static;

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
    Ticking,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range : i32, item : Entity},
    ShowRemoveItem,
    MainMenu { menu_selection : gui::MainMenuSelection },
    GameOver,
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
        //Kill particles
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        //draw
        match newrunstate {
            RunState::MainMenu{..} => {}
            _ => {
                //draw
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }
        
        match newrunstate {
            RunState::MainMenu{ .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
                    gui::MainMenuResult::Selected{ selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            //gui::MainMenuSelection::LoadGame => newrunstate = RunState::PreRun,
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                //makes sure used items are removed
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::Ticking => {
                self.run_systems();
                //makes sure used items are removed
                self.ecs.maintain();

                match *self.ecs.fetch::<RunState>() {
                    RunState::AwaitingInput => newrunstate = RunState::AwaitingInput,
                    _ => newrunstate = RunState::Ticking
                }   
            }
            // RunState::MonsterTurn => {
            //     self.run_systems();
            //     //makes sure used items are removed
            //     self.ecs.maintain();
            //     newrunstate = RunState::AwaitingInput;
            // }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();

                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting{ range: is_item_ranged.range, item: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item: item_entity, target: None }).expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowTargeting{range, item} => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: result.1 }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToRemoveItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
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
    }
}

//the meat of the EC*S*
impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut initiative = ai::InitiativeSystem{};
        initiative.run_now(&self.ecs);
        //this determines who gets to act, so needs to run before main AI
        let mut turnstatus = ai::TurnStatusSystem{};
        turnstatus.run_now(&self.ecs);
        let mut quipper = ai::QuipSystem{};
        quipper.run_now(&self.ecs);
        //needs to run before main AI
        let mut adjacent = ai::AdjacentAI{};
        adjacent.run_now(&self.ecs);
        let mut visible = ai::VisibleAI{};
        visible.run_now(&self.ecs);
        let mut approach = ai::ApproachAI{};
        approach.run_now(&self.ecs);
        let mut flee = ai::FleeAI{};
        flee.run_now(&self.ecs);
        //indexing needs to run after AI and before combat, so that combat knows the new positions
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut defaultmove = ai::DefaultMoveAI{};
        defaultmove.run_now(&self.ecs);
        //needs to go before combat, because it can deal damage too
        let mut triggers = trigger_system::TriggerSystem{};
        triggers.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        //items
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut items = ItemUseSystem{};
        items.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);
        let mut item_remove = ItemRemoveSystem{};
        item_remove.run_now(&self.ecs);
        //goes last because nearly anything can in theory produce one of those
        let mut particles = particle_system::ParticleSpawnSystem{};
        particles.run_now(&self.ecs);
        let mut lighting = lighting_system::LightingSystem{};
        lighting.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl State {
    fn generate_world(&mut self) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
        let mut builder = map_builders::random_builder(&mut rng, 80, 60); //80,60 usually unless testing
        console::log("Generating world...");
        builder.build_map(&mut rng);

        //prevent borrow checker errors
        std::mem::drop(rng);
        //mapgen visualizer data
        self.mapgen_history = builder.build_data.history.clone();
        //key stuff
        self.ecs.insert(RunState::MapGeneration);
        //self.ecs.insert(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame});
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

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }
    
        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }
    
        // Build a new map and place the player
        self.generate_world();                                          
    }
}

// Auto-starts on page load
//start section of the executable may not literally point to main
//#[wasm_bindgen(start)]
//can't use wasm_bindgen(start) because RLTK-rs uses it
pub fn main() {

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    let mut context = Rltk::init_simple8x8(80, 60, "RLTK Web roguelike", "resources");
    context.with_post_scanlines(true);

    //ECS takes more lines to set up
    let mut gs = State {
        ecs: World::new(),
        //show main menu
        mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame}),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Bystander>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<MedItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<Player>();

    //load JSON data
    raws::load_raws();

    //placeholders so that generate_world has stuff to fill
    gs.ecs.insert(Map::new(80,50));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    gs.generate_world();

    gs.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Neon Twilight!".to_string()] });
    gs.ecs.insert(particle_system::ParticleBuilder::new());

    //register html buttons
    rltk::register_html_button("go_nw");
    rltk::register_html_button("go_n");
    rltk::register_html_button("go_ne");
    rltk::register_html_button("go_w");
    rltk::register_html_button("go_e");
    rltk::register_html_button("go_sw");
    rltk::register_html_button("go_s");
    rltk::register_html_button("go_se");
    rltk::register_html_button("go_wait");
    //non-movement
    rltk::register_html_button("confirm");
    rltk::register_html_button("get");
    rltk::register_html_button("inven");
    rltk::register_html_button("drop");
    rltk::register_html_button("remove");
    //inventory
    rltk::register_html_button("escape");
    rltk::register_html_button("a");
    rltk::register_html_button("b");
    rltk::register_html_button("c");
    rltk::register_html_button("d");


    rltk::main_loop(context, gs);
} 
