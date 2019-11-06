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
    pub history : Vec<Map>
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
    pub fn new() -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            build_data : BuilderMap {
                list_spawns: Vec::new(),
                map: Map::new(),
                starting_position: None,
                rooms: None,
                history : Vec::new()
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

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}

pub trait MetaMapBuilder {    
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}


//Rust's interface - unfortunately, no variables allowed here!
pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&self) -> Position;
    //mapgen visualizer
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    //for spawning with multiple layers
    fn get_list_spawns(&self) -> &Vec<(usize, String)>;
    //default implementation
    fn spawn_entities(&mut self, ecs : &mut World) {
        for entity in self.get_list_spawns().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }

}


//Factory function for builder
pub fn random_builder(rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    //let mut builder = BuilderChain::new();
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

    //show off
    let mut builder = BuilderChain::new();
    builder.start_with(VoronoiBuilder::pythagoras());
    //builder.with(PrefabBuilder::vaults());
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    //builder.with(CullUnreachable::new()); culling not implemented yet
    builder.with(VoronoiSpawning::new());
    builder.with(PrefabBuilder::sectional(prefab_builders::prefab_sections::UNDERGROUND_FORT));
    builder
}


 //Box::new(SimpleMapBuilder::new())
    //Box::new(BSPDungeonBuilder::new())
    //Box::new(CellularAutomataBuilder::new())
    // three variants of the drunkard walk algo
    //Box::new(DrunkardsWalkBuilder::open_area())
    //Box::new(DrunkardsWalkBuilder::open_halls())
    //Box::new(DrunkardsWalkBuilder::winding_passages())
    // //custom one
    // Box::new(DrunkardsWalkBuilder::new(DrunkardSettings{ 
    //     spawn_mode: DrunkSpawnMode::Random,
    //     drunken_lifetime: 100,
    //     floor_percent: 0.4,
    //     brush_size: 1,
    //     symmetry: Symmetry::None
    //     }))
    //Box::new(MazeBuilder::new())
    //Box::new(DLABuilder::walk_outwards())
    //Box::new(DLABuilder::insectoid())
    //Box::new(DrunkardsWalkBuilder::fat_passages())
    //Box::new(DrunkardsWalkBuilder::fearful_symmetry())
    //Box::new(VoronoiBuilder::pythagoras())
    //Box::new(VoronoiBuilder::manhattan())
    //Box::new(PrefabBuilder::new())
    //this one is sectional
    // Box::new(
    //     PrefabBuilder::new(
    //         Some(Box::new(CellularAutomataBuilder::new()))
    //     )
    // )