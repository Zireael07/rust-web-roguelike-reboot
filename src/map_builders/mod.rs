use super::{Map, TileType, Rect, Position, spawner};

mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;
use specs::prelude::*;

//Rust's interface
trait MapBuilder {
    //Position is the player start position
    //it can be overridden by each of various map builders
    fn build() -> (Map, Position);
    fn spawn(map : &mut Map, ecs : &mut World);
}

//Public functions for separate builders
pub fn build_random_map() -> (Map, Position) {
    SimpleMapBuilder::build()
}

pub fn spawn(map : &mut Map, ecs : &mut World) {
    SimpleMapBuilder::spawn(map, ecs);
}