use super::{Map, TileType, Rect, Position};

mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;

//Rust's interface
trait MapBuilder {
    //Position is the player start position
    //it can be overridden by each of various map builders
    fn build() -> (Map, Position);
}

//Public functions for separate builders
pub fn build_random_map() -> (Map, Position) {
    SimpleMapBuilder::build()
}