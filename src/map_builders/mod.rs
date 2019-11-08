use super::{Map, TileType, Rect, Position, spawner, SHOW_MAPGEN_VISUALIZER};

mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BSPDungeonBuilder;
mod bsp_interior;
use bsp_interior::BSPInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod drunkard_walk;
use drunkard_walk::*;
mod maze;
use maze::MazeBuilder;
mod dla;
use dla::*;
mod voronoi;
use voronoi::*;
mod prefab_builders;
use prefab_builders::*;

//postprocessing stuff
mod room_based_spawner;
use room_based_spawner::RoomBasedSpawner;
mod room_based_starting;
use room_based_starting::RoomBasedStartingPosition;
mod room_eroder;
use room_eroder::RoomEroder;
mod room_corner_rounding;
use room_corner_rounding::RoomCornerRounder;
mod area_starting_points;
use area_starting_points::*;
mod voronoi_spawning;
use voronoi_spawning::VoronoiSpawning;

mod common;
use common::*;
use specs::prelude::*;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

pub struct BuilderMap {
    pub list_spawns : Vec<(usize, String)>,
    pub map : Map,
    pub starting_position : Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history : Vec<Map>,
    pub width: i32,
    pub height: i32
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data : BuilderMap
}

impl BuilderChain {
    pub fn new(width: i32, height: i32) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            build_data : BuilderMap {
                list_spawns: Vec::new(),
                map: Map::new(width, height),
                starting_position: None,
                rooms: None,
                history : Vec::new(),
                width,
                height
            }
        }
    }

    pub fn start_with(&mut self, starter : Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder.")
        };
    }

    //for chaining metabuilders
    pub fn with(&mut self, metabuilder : Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs : &mut World) {
        for entity in self.build_data.list_spawns.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

//Rust's interfaces - unfortunately, no variables allowed here!
pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}

pub trait MetaMapBuilder {    
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}


//Factory function for builder
pub fn random_builder(rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    //let mut builder = BuilderChain::new(width, height);
    // //simple map
    // //builder.start_with(SimpleMapBuilder::new());
    // builder.start_with(BSPDungeonBuilder::new());
    // builder.with(RoomBasedSpawner::new());
    // builder.with(RoomBasedStartingPosition::new());
    // builder

    // cellular
    //builder.start_with(CellularAutomataBuilder::new());
    //builder.start_with(DrunkardsWalkBuilder::fearful_symmetry());
    //builder.start_with(DLABuilder::insectoid());
    //builder.start_with(MazeBuilder::new());
    // builder.start_with(VoronoiBuilder::pythagoras());
    // builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    // //builder.with(CullUnreachable::new()); culling not implemented yet
    // builder.with(VoronoiSpawning::new());
    // builder

    //another example
    let mut builder = BuilderChain::new(width, height);
    builder.start_with(SimpleMapBuilder::new());
    //builder.with(DrunkardsWalkBuilder::winding_passages());
    //builder.with(DLABuilder::heavy_erosion());
    //builder.with(RoomEroder::new());
    builder.with(RoomCornerRounder::new());
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    //builder.with(CullUnreachable::new()); culling not implemented yet
    builder.with(VoronoiSpawning::new());
    builder


    //show off
    // let mut builder = BuilderChain::new(width, height);
    // builder.start_with(NoiseMapBuilder::new());
    // //builder.start_with(VoronoiBuilder::pythagoras());
    // //builder.with(CellularAutomataBuilder::new());
    // //builder.with(PrefabBuilder::vaults());
    // builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    // //builder.with(CullUnreachable::new()); culling not implemented yet
    // builder.with(VoronoiSpawning::new());
    // builder.with(PrefabBuilder::sectional(prefab_builders::prefab_sections::UNDERGROUND_FORT));
    //builder
}
