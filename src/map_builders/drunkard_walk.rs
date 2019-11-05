use super::{MapBuilder, Map,  
    TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER,
    remove_unreachable_areas_returning_most_distant, generate_voronoi_spawn_regions,
    Symmetry, paint};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode { StartingPoint, Random }

pub struct DrunkardSettings {
    pub spawn_mode : DrunkSpawnMode,
    pub drunken_lifetime : i32,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry
}

pub struct DrunkardsWalkBuilder {
    map : Map,
    starting_position : Position,
    history: Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>,
    settings : DrunkardSettings,
    list_spawns: Vec<(usize, String)>
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self)  {
        self.build();
    }

    fn get_list_spawns(&self) -> &Vec<(usize, String)> {
        &self.list_spawns
    }

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

impl DrunkardsWalkBuilder {
    pub fn new(settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings,
            list_spawns: Vec::new()
        }
    }

    //preset constructors
    pub fn open_area() -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None
            },
            list_spawns: Vec::new()
        }
    }

    pub fn open_halls() -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None
            },
            list_spawns: Vec::new()
        }
    }

    pub fn winding_passages() -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None
            },
            list_spawns: Vec::new()
        }
    }

    pub fn fat_passages() -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None
            },
            list_spawns: Vec::new()
        }
    }

    pub fn fearful_symmetry() -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history: Vec::new(),
            noise_areas : HashMap::new(),
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both
            },
            list_spawns: Vec::new()
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Set a central starting point
        self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize; //the default was 0.5
        let mut floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        let mut digger_count = 0;
        let mut active_digger_count = 0;
        //keep working until we have reached the target percentage of floor tiles
        while floor_tile_count  < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = self.starting_position.x;
                    drunk_y = self.starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = self.starting_position.x;
                        drunk_y = self.starting_position.y;
                    } else {
                        drunk_x = rng.roll_dice(1, self.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    }
                }
            }
            let mut drunk_life = self.settings.drunken_lifetime;

            while drunk_life > 0 {
                let drunk_idx = self.map.xy_idx(drunk_x, drunk_y);
                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(&mut self.map, self.settings.symmetry, self.settings.brush_size, drunk_x, drunk_y);
                self.map.tiles[drunk_idx] = TileType::DownStairs; //debug

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => { if drunk_x > 2 { drunk_x -= 1; } }
                    2 => { if drunk_x < self.map.width-2 { drunk_x += 1; } }
                    3 => { if drunk_y > 2 { drunk_y -=1; } }
                    _ => { if drunk_y < self.map.height-2 { drunk_y += 1; } }
                }

                drunk_life -= 1;
            }
            if did_something { 
                self.take_snapshot(); 
                active_digger_count += 1;
            }

            digger_count += 1;
            for t in self.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
       console::log(&format!("{} dwarves gave up their sobriety, of whom {} actually found a wall.", digger_count, active_digger_count));


        // Find all tiles we can reach from the starting point
        //let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // Spawn the entities
        for area in self.noise_areas.iter() {
            spawner::spawn_region(&self.map, &mut rng, area.1, &mut self.list_spawns);
        }
    }
}