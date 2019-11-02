use super::{Map, TileType, Rect, Position, spawner, SHOW_MAPGEN_VISUALIZER};

mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BSPDungeonBuilder;
mod bsp_interior;
use bsp_interior::BSPInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod common;
use common::*;
use specs::prelude::*;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

//Rust's interface - unfortunately, no variables allowed here!
pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs : &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&self) -> Position;
    //mapgen visualizer
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

//Factory function for builder
pub fn random_builder() -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slightly random
    //Box::new(SimpleMapBuilder::new())
    //console::log("Simple map builder!");
    //Box::new(BSPDungeonBuilder::new())
    Box::new(CellularAutomataBuilder::new())
}