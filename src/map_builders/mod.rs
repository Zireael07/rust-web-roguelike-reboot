use super::{Map, TileType, Rect};

mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;

//Rust's interface
trait MapBuilder {
    fn build() -> Map;
}

//Public functions for separate builders
pub fn build_random_map() -> Map {
    SimpleMapBuilder::build()
}