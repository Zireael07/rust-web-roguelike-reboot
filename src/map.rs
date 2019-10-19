use rltk::{ RGB, Rltk, Console, RandomNumberGenerator };
use super::{Rect};
use std::cmp::{max, min};
extern crate specs;
use specs::prelude::*;

// We'll allow map tiles to be either a wall or a floor. We're deriving PartialEq so we don't
// have to match on it every time. We'll make it a copy type because it's really just an int.
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32
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

    fn apply_room_to_map(&mut self, room : &Rect) {
        for y in room.y1 +1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80*50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80*50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    //Generator needs to return the room list
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map{
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height: 50
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);        

                //connect rooms
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,1) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);            
            }
        }

        map
    }
}

pub fn draw_map(ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();
    // Iterate the map array, incrementing coordinates as we go.
    let mut y = 0;
    let mut x = 0;
    for tile in map.tiles.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.print_color(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    ".",
                );
            }
            TileType::Wall => {
                ctx.print_color(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    "#",
                );
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}