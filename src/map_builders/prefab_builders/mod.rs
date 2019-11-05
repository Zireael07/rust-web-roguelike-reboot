use super::{MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
mod prefab_levels;
mod prefab_sections;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

//under the hood, Rust enumerations are actually unions. They can hold whatever you want to put in there
#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum PrefabMode { 
    RexLevel{ template : &'static str },
    Constant{ level : prefab_levels::PrefabLevel },
    Sectional{ section : prefab_sections::PrefabSection }
}

pub struct PrefabBuilder {
    map : Map,
    starting_position : Position,
    history: Vec<Map>,
    mode: PrefabMode,
    //because sections take a completed map
    previous_builder : Option<Box<dyn MapBuilder>>
}

impl MapBuilder for PrefabBuilder {
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

    fn spawn_entities(&mut self, ecs : &mut World) {
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

impl PrefabBuilder {
    pub fn new(previous_builder : Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            history : Vec::new(),
            mode : PrefabMode::Sectional{ section: prefab_sections::UNDERGROUND_FORT },
            previous_builder
        }
    }

    fn build(&mut self) {
        match self.mode {
            //makes template available in match scope
            PrefabMode::RexLevel{template} => self.load_rex_map(&template),
            PrefabMode::Constant{level} => self.load_ascii_map(&level),
            PrefabMode::Sectional{section} => self.apply_sectional(&section)
        }
        self.take_snapshot();

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }
        self.take_snapshot();
    }

    fn char_to_map(&mut self, ch : char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor, //space
            '#' => self.map.tiles[idx] = TileType::Wall, // #
            _ => {
                //put a floor
                self.map.tiles[idx] = TileType::Floor;
                //log an error
                console::log(&format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
    }

    fn read_ascii_to_vec(template : &str) -> Vec<char> {
        // Start by converting to a vector, with newlines removed
        // the filter function does the stripping
        let mut string_vec : Vec<char> = template.chars().filter(|a| *a != '\r' && *a !='\n').collect();
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' '; } }
        string_vec
    }

    #[allow(dead_code)]
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {

        let mut string_vec = PrefabBuilder::read_ascii_to_vec(level.template);

        let mut i = 0;
        for ty in 0..level.height {
            //fix off by one
            for tx in 0..level.width-1 {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    //paranoia
                    if i < string_vec.len() {
                        self.char_to_map(string_vec[i], idx);
                    }
                }
                i += 1;
            }
        }
    }

    pub fn apply_sectional(&mut self, section : &prefab_sections::PrefabSection) {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        //copy starting position and map from previous
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();
        self.take_snapshot();

        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);
        
        // Place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (self.map.width-1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height-1) - section.height as i32
        }
        console::log(&format!("{},{}", chunk_x, chunk_y));

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }



    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }
}