use super::{InitialMapBuilder, BuilderMap, Rect, TileType };
use rltk::RandomNumberGenerator;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

const MIN_ROOM_SIZE : i32 = 8;

pub struct BSPTownBuilder {
    rects: Vec<Rect>
}

impl InitialMapBuilder for BSPTownBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BSPTownBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<BSPTownBuilder> {
        Box::new(BSPTownBuilder{
            rects: Vec::new()
        })
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut rooms : Vec<Rect> = Vec::new();

        //fill with floors
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::Floor;
            }
        }
        build_data.take_snapshot();


        //place walls around
        //Rust is weird, ranges are inclusive at the beginning but exclusive at the end
        for x in 0 ..build_data.map.width{
            let mut idx = build_data.map.xy_idx(x, 1);
            build_data.map.tiles[idx] = TileType::Wall;
            idx = build_data.map.xy_idx(x, build_data.map.height-2);
            build_data.map.tiles[idx] = TileType::Wall;
        }
        for y in 0 ..build_data.map.height{
            let mut idx = build_data.map.xy_idx(1, y);
            build_data.map.tiles[idx] = TileType::Wall;
            idx = build_data.map.xy_idx(build_data.map.width-2, y);
            build_data.map.tiles[idx] = TileType::Wall;
        }

        build_data.take_snapshot();

        //BSP now
        self.rects.clear();
        self.rects.push( Rect::new(1, 1, build_data.map.width-2, build_data.map.height-2) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(rng);

            //stop too small
            let rect_width = i32::abs(rect.x1 - rect.x2);
            let rect_height = i32::abs(rect.y1 - rect.y2);
            if rect_width > MIN_ROOM_SIZE && rect_height > MIN_ROOM_SIZE { 
                let candidate = self.get_random_sub_rect(rect, rng);
                //console::log(format!("rect candidate: {:?}", candidate));

                if self.is_possible(candidate, &build_data, &rooms) {
                    rooms.push(candidate);
                    self.add_subrects(rect);
                    //buildings added further on
                }
            }

            n_rooms += 1;
        }



        //let rooms_copy = self.rects.clone();
        let rooms_copy = rooms.clone();
        for r in rooms_copy.iter() {
            let room = *r;
            //rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0 && idx < ((build_data.map.width * build_data.map.height)-1) as usize {
                        build_data.map.tiles[idx] = TileType::Wall;
                    }
                }
            }
            //build_data.take_snapshot();

            for y in room.y1+1 .. room.y2-1 {
                for x in room.x1+1 .. room.x2-1 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0 && idx < ((build_data.map.width * build_data.map.height)-1) as usize {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();

            //build doors
            let cent = room.center();
            let door_direction = rng.roll_dice(1, 4);
            match door_direction {
                1 => { 
                    let idx = build_data.map.xy_idx(cent.0, room.y1); //north
                    build_data.map.tiles[idx] = TileType::Floor;
                }
                2 => { 
                    let idx = build_data.map.xy_idx(cent.0, room.y2-1); //south
                    build_data.map.tiles[idx] = TileType::Floor;
                }
                3 => { 
                    let idx = build_data.map.xy_idx(room.x1, cent.1); //west
                    build_data.map.tiles[idx] = TileType::Floor;
                }
                _ => { 
                    let idx = build_data.map.xy_idx(room.x2-1, cent.1); //east
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }

    //taken from BSP dungeon...
    //BSP subdivision happens here
    fn add_subrects(&mut self, rect : Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects.push(Rect::new( rect.x1, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1, rect.y1 + half_height, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1 + half_height, half_width, half_height ));
    }

    //helpers
    fn get_random_rect(&mut self, rng : &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect : Rect, rng : &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        //let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
        //let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;
        let w = rng.roll_dice(2,4)+4;
        let h = rng.roll_dice(2,4)+4;

        result.x1 += rng.roll_dice(2, 4);
        result.y1 += rng.roll_dice(2, 4);
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect : Rect, build_data : &BuilderMap, rooms: &Vec<Rect>) -> bool {
        //expanding prevents overlapping rooms
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for r in rooms.iter() {
            if r.intersect(&rect) { 
                can_build = false; 
                console::log(&format!("Candidate {:?} overlaps a room {:?}", rect, r));
            }
        }

        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                if x > build_data.map.width-2 { can_build = false; }
                if y > build_data.map.height-2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                if can_build {
                    let idx = build_data.map.xy_idx(x, y);
                    if build_data.map.tiles[idx] != TileType::Floor { //key change
                        console::log(&format!("Candidate {:?} failed the tile check!", rect));
                        can_build = false; 
                    }
                }
            }
        }

        can_build
    }
}