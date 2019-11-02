use super::{MapBuilder, Map, TileType, common, Rect, apply_room_to_map, Position,
    spawner, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

const MIN_ROOM_SIZE : i32 = 8;

pub struct BSPInteriorBuilder {
    map : Map,
    starting_position : Position,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>
}

impl MapBuilder for BSPInteriorBuilder {
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
        //we skip room 1 because we don't want any in starting room
        for room in self.rooms.iter().skip(1) {
            //let (x,y) = room.center();
            //spawner::random_monster(ecs, x, y);
            spawner::spawn_room(ecs, room);
        }
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

impl BSPInteriorBuilder {
    pub fn new() -> BSPInteriorBuilder {
        BSPInteriorBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new()
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects.push( Rect::new(1, 1, self.map.width-2, self.map.height-2) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room, &mut rng); // Divide the first room

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            let room = *r;
            //room.x2 -= 1;
            //room.y2 -= 1;
            self.rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = self.map.xy_idx(x, y);
                    if idx > 0 && idx < ((self.map.width * self.map.height)-1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            self.take_snapshot();
        }

        // Now we want corridors
        for i in 0..self.rooms.len()-1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
        }

        let start = self.rooms[0].center();
        self.starting_position = Position{ x: start.0, y: start.1 };
    }

    //BSP subdivision, horizontal or vertical
    fn add_subrects(&mut self, rect : Rect, rng : &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Calculate boundaries
        let width  = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        let half_width = width / 2;
        let half_height = height / 2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new( rect.x1, rect.y1, half_width-1, height );
            self.rects.push( h1 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h1, rng); }
            let h2 = Rect::new( rect.x1 + half_width, rect.y1, half_width, height );
            self.rects.push( h2 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h2, rng); }
        } else {
            // Vertical split
            let v1 = Rect::new( rect.x1, rect.y1, width, half_height-1 );
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v1, rng); }
            let v2 = Rect::new( rect.x1, rect.y1 + half_height, width, half_height );
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v2, rng); }
        }
    }

    //very simple corridors
    fn draw_corridor(&mut self, x1:i32, y1:i32, x2:i32, y2:i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }

            let idx = self.map.xy_idx(x, y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}