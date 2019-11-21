use super::{MetaMapBuilder, BuilderMap, TileType };
use rltk::RandomNumberGenerator;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    #[allow(dead_code)]
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement{ })
    }

    //helper
    fn is_floor(&self, tt : TileType) -> bool {
        match tt {
            TileType::Floor | TileType::FloorIndoor
                => true,
            _ => false        
        }
    }

    fn door_possible(&self, build_data : &mut BuilderMap, idx : usize) -> bool {
        let x = (idx % build_data.map.width as usize) as i32;
        let y = (idx / build_data.map.width as usize) as i32;
    
        // Check for east-west door possibility
        // north/south = wall, east/west = (indoor) floor
        if build_data.map.tiles[idx] == TileType::Floor &&
            (x > 1 && self.is_floor(build_data.map.tiles[idx-1])) &&
            (x < build_data.map.width-2 && self.is_floor(build_data.map.tiles[idx+1])) &&
            (y > 1 && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall) &&
            (y < build_data.map.height-2 && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall)
        {
            return true;
        }
    
        // Check for north-south door possibility
        // east/west = wall, north/south = (indoor) floor
        if build_data.map.tiles[idx] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[idx-1] == TileType::Wall) &&
            (x < build_data.map.width-2 && build_data.map.tiles[idx+1] == TileType::Wall) &&
            (y > 1 && self.is_floor(build_data.map.tiles[idx - build_data.map.width as usize])) &&
            (y < build_data.map.height-2 && self.is_floor(build_data.map.tiles[idx + build_data.map.width as usize]))
        {
            return true;
        }
    
        false
    }

    fn doors(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone(); // To avoid nested borrowing
            for hall in halls.iter() {
                if hall.len() > 2 { // We aren't interested in tiny corridors
                    if self.door_possible(build_data, hall[0]) {
                        build_data.list_spawns.push((hall[0], "Door".to_string()));
                    }
                }
            }
        } else {        
            // There are no corridors - scan for possible places
            let tiles = build_data.map.tiles.clone();
            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor && self.door_possible(build_data, i) {
                    build_data.list_spawns.push((i, "Door".to_string()));
                }
            }
        }
    }
}