use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, TileType, Position};
use rltk::RandomNumberGenerator;
//these need to be accessible on the outside
pub mod prefab_levels;
pub mod prefab_sections;
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
    mode: PrefabMode
}

//it implements two interfaces at once, as it can be both the initial and the meta builder
impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn constant(level : prefab_levels::PrefabLevel) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::Constant{ level },
        })
    }

    #[allow(dead_code)]
    pub fn sectional(section : prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::Sectional{ section },
        })
    }

    #[allow(dead_code)]
    pub fn rex_level(template : &'static str) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::RexLevel{ template },
        })
    }


    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        match self.mode {
            //makes template available in match scope
            PrefabMode::RexLevel{template} => self.load_rex_map(&template, build_data),
            PrefabMode::Constant{level} => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional{section} => self.apply_sectional(&section, rng, build_data)
        }
        build_data.take_snapshot();

        //spawning handled by post-process
    }

    fn char_to_map(&mut self, ch : char, idx: usize, build_data : &mut BuilderMap) {
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor, //space
            '#' => build_data.map.tiles[idx] = TileType::Wall, // #
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.list_spawns.push((idx, "Human".to_string()));
            }
            _ => {
                //put a floor
                build_data.map.tiles[idx] = TileType::Floor;
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
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data : &mut BuilderMap) {

        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);

        let mut i = 0;
        for ty in 0..level.height {
            //fix off by one
            for tx in 0..level.width-1 {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);
                    //paranoia
                    if i < string_vec.len() {
                        self.char_to_map(string_vec[i], idx, build_data);
                    }
                }
                i += 1;
            }
        }
    }

    //generic: F is a template
    fn apply_previous_iteration<F>(&mut self, mut filter: F, _rng: &mut RandomNumberGenerator, build_data : &mut BuilderMap)
        //requiring x and y (i32) and the spawn format - map index (usize) and String
        where F : FnMut(i32, i32, &(usize, String)) -> bool
    {
        //spawning from previous list [of] spawns
        let spawn_clone = build_data.list_spawns.clone();
        for e in spawn_clone.iter() {
            let idx = e.0;
            let x = idx as i32 % build_data.map.width;
            let y = idx as i32 / build_data.map.width;
            if filter(x, y, e) {
                build_data.list_spawns.push(
                    (idx, e.1.to_string())
                )
            }
        }        
        build_data.take_snapshot(); 
    }

    pub fn apply_sectional(&mut self, section : &prefab_sections::PrefabSection, rng: &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);
        
        // Place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (build_data.map.width-1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (build_data.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (build_data.map.height-1) - section.height as i32
        }
        console::log(&format!("{},{}", chunk_x, chunk_y));

        // Build the map
        self.apply_previous_iteration(|x,y,_e| {
            x < chunk_x || x > (chunk_x + section.width as i32) || y < chunk_y || y > (chunk_y + section.height as i32)
        }, rng, build_data);

        //place section
        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }


    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str, build_data : &mut BuilderMap) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx, build_data);
                    }
                }
            }
        }
    }
}