use rltk::{ RGB, Rltk, Console, BaseMap, Algorithm2D, Point };
use std::cmp::{max, min};
extern crate specs;
use specs::prelude::*;

// We'll allow map tiles to be either a wall or a floor. We're deriving PartialEq so we don't
// have to match on it every time. We'll make it a copy type because it's really just an int.
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

//After the refactor, this is just the data structure
//Map building is done by the map_builders module
//Default is to create an empty instance
#[derive(Default, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>
}

impl Map {
    // We're storing all the tiles in one big array, so we need a way to map an X,Y coordinate to
    // a tile. Each row is stored sequentially (so 0..80, 81..160, etc.). This takes an x/y and returns
    // the array index.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * 80) + x as usize
    }

    // It's a great idea to have a reverse mapping for these coordinates. This is as simple as
    // index % 80 (mod 80), and index / 80
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % 80, idx as i32 / 80)
    }

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new() -> Map {
        Map{
            tiles : vec![TileType::Wall; 80*50],
            width : 80,
            height: 50,
            revealed_tiles : vec![false; 80*50],
            visible_tiles : vec![false; 80*50],
        }
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
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, _idx:i32) -> Vec<(i32, f32)> {
        Vec::new()
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

//the tutorial checked for revealed also, but that led to pillars mysteriously transforming
fn is_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall //&& map.revealed_tiles[idx]
}

fn wall_glyph(map : &Map, x: i32, y:i32) -> u8 {
    if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
    let mut mask : u8 = 0;

    
    //bitmask is here
    if is_wall(map, x, y - 1) { mask +=1; }
    if is_wall(map, x, y + 1) { mask +=2; }
    if is_wall(map, x - 1, y) { mask +=4; }
    if is_wall(map, x + 1, y) { mask +=8; }

    match mask {
        0 => { 9 } // Pillar because no neighbors
        1 => { 186 } // Wall only to the north
        2 => { 186 } // Wall only to the south
        3 => { 186 } // Wall to the north and south
        4 => { 205 } // Wall only to the west
        5 => { 188 } // Wall to the north and west
        6 => { 187 } // Wall to the south and west
        7 => { 185 } // Wall to the north, south and west
        8 => { 205 } // Wall only to the east
        9 => { 200 } // Wall to the north and east
        10 => { 201 } // Wall to the south and east
        11 => { 204 } // Wall to the north, south and east
        12 => { 205 } // Wall to the east and west
        13 => { 202 } // Wall to the east, west, and south
        14 => { 203 } // Wall to the east, west, and north
        _ => { 35 } // We missed one?
    }
}

pub fn draw_map(map : &Map, ctx : &mut Rltk) {
    // Iterate the map array, incrementing coordinates as we go.
    let mut y = 0;
    let mut x = 0;
    for (idx,tile) in map.tiles.iter().enumerate() {
        // Render only revealed tiles
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            // Render a tile depending upon the tile type
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    //glyph = rltk::to_cp437('#');
                    glyph = wall_glyph(&*map, x, y);
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
            }
            //grayscale out of FOV
            if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
            // draw
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}