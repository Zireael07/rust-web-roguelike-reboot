use rltk::{ RGB, Rltk, Console, BaseMap, Algorithm2D, Point };
use std::cmp::{max, min};
extern crate specs;
use specs::prelude::*;
use std::collections::HashSet;

// We'll allow map tiles to be either a wall or a floor. We're deriving PartialEq so we don't
// have to match on it every time. We'll make it a copy type because it's really just an int.
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Tree,
    Floor,
    FloorIndoor,
    DownStairs,
}

//After the refactor, this is just the data structure
//Map building is done by the map_builders module
//Default is to create an empty instance
#[derive(Default, Clone)]
pub struct Map {
    pub width : i32,
    pub height : i32,
    pub tiles : Vec<TileType>,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub view_blocked : HashSet<usize>,
    pub light : Vec<rltk::RGB>,
    pub tile_content : Vec<Vec<Entity>>
}

impl Map {
    // We're storing all the tiles in one big array, so we need a way to map an X,Y coordinate to
    // a tile. Each row is stored sequentially (so 0..80, 81..160, etc.). This takes an x/y and returns
    // the array index.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    // It's a great idea to have a reverse mapping for these coordinates. This is as simple as
    // index % self.width (mod self.width), and index / self.width
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new(width: i32, height: i32) -> Map {
        let map_count = (width*height) as usize;
        Map{
            width,
            height,
            tiles : vec![TileType::Wall; map_count],
            revealed_tiles : vec![false; map_count],
            visible_tiles : vec![false; map_count],
            blocked : vec![false; map_count],
            view_blocked : HashSet::new(),
            light: vec![rltk::RGB::from_f32(0.0, 0.0, 0.0); map_count],
            tile_content : vec![Vec::new(); map_count],
        }
    }

    //for indexing
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i,tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = !tile_walkable(*tile);
        }
    }

    //used by pathfinding
    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        //bounds check
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = (y * self.width) + x;
        !self.blocked[idx as usize]
    }

}

    /// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
    /// look awful.
    // fn new_map_test() -> Vec<TileType> {
    //     let mut map = vec![TileType::Floor; 80 * 50];

    //     // Make the boundaries walls
    //     for x in 0..80 {
    //         map[self.xy_idx(x, 0)] = TileType::Wall;
    //         map[self.xy_idx(x, 49)] = TileType::Wall;
    //     }
    //     for y in 0..50 {
    //         map[self.xy_idx(0, y)] = TileType::Wall;
    //         map[self.xy_idx(79, y)] = TileType::Wall;
    //     }

    //     // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    //     // First, obtain the thread-local RNG:
    //     let mut rng = rltk::RandomNumberGenerator::new();

    //     for _ in 0..400 {
    //         // rand provides a gen_range function to get numbers in a range.
    //         let x = rng.roll_dice(1, 80) - 1;
    //         let y = rng.roll_dice(1, 50) - 1;
    //         let idx = self.xy_idx(x, y);
    //         // We don't want to add a wall on top of the player
    //         if idx != self.xy_idx(40,25) {
    //             map[idx] = TileType::Wall;
    //         }
    //     }

    //     // We'll return the state with the short-hand
    //     map
    // }


//implementing RLTK traits
impl BaseMap for Map {
    fn is_opaque(&self, idx:i32) -> bool {
        let idx_u = idx as usize;
        if idx > 0 {
            return tile_opaque(self.tiles[idx_u]) || self.view_blocked.contains(&idx_u);
        }
        //paranoia just in case
        else {
            return true;
        }
    }

    //for pathfinding
    fn get_available_exits(&self, idx:i32) -> Vec<(i32, f32)> {
        let mut exits : Vec<(i32, f32)> = Vec::new();
        let x = idx % self.width;
        let y = idx / self.width;
    
        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-self.width, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+self.width, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-self.width)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-self.width)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+self.width)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+self.width)+1, 1.45)); }
    
        exits
    }

    fn get_pathing_distance(&self, idx1:i32, idx2:i32) -> f32 {
        let p1 = Point::new(idx1 % self.width, idx1 / self.width);
        let p2 = Point::new(idx2 % self.width, idx2 / self.width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn point2d_to_index(&self, pt: Point) -> i32 {
        (pt.y * self.width) + pt.x
    }

    fn index_to_point2d(&self, idx:i32) -> Point {
        Point{ x: idx % self.width, y: idx / self.width }
    }
}

//helpers
pub fn tile_walkable(tt : TileType) -> bool {
    match tt {
        TileType::Floor | TileType::FloorIndoor | TileType::DownStairs 
            => true,
        _ => false        
    }
}

pub fn tile_opaque(tt : TileType) -> bool {
    match tt {
        TileType::Wall | TileType::Tree 
            => true,
        _ => false
    }
}