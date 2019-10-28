use super::{MapBuilder, Map, common, Rect, apply_room_to_map, 
    apply_horizontal_tunnel, apply_vertical_tunnel, Position,
    spawner};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct SimpleMapBuilder {
    map : Map,
    starting_position : Position,
    //specific to this builder
    rooms: Vec<Rect>
}

impl MapBuilder for SimpleMapBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, map : &mut Map, ecs : &mut World) {
        //we skip room 1 because we don't want any in starting room
        for room in self.rooms.iter().skip(1) {
            let (x,y) = room.center();
            spawner::random_monster(ecs, x, y);
        }
    }
}

impl SimpleMapBuilder {
    pub fn new() -> SimpleMapBuilder {
        SimpleMapBuilder{
            map : Map::new(),
            starting_position : Position{ x: 0, y : 0 },
            rooms: Vec::new()
        }
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                apply_room_to_map(&mut self.map, &new_room);        

                //connect rooms
                if !self.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms[self.rooms.len()-1].center();
                    if rng.range(0,1) == 1 {
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                    }
                }

                self.rooms.push(new_room);            
            }
        }

        let start_pos = self.rooms[0].center();
        self.starting_position = Position{ x: start_pos.0, y: start_pos.1 }
    }
}