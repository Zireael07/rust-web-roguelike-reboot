use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, Rect, TileType, Position };
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

impl MetaMapBuilder for BSPTownBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        //meta version panics if no submaps
        let submaps : Vec<Rect>;
        if let Some(submaps_builder) = &build_data.submaps {
            submaps = submaps_builder.clone();
        } else {
            panic!("Using BSP town as meta requires a builder with submap structures");
        }

        self.build(rng, build_data);
    }
}

#[derive(Debug)]
enum BuildingTag {
    Pub,
    Hovel,
    Unassigned,
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

        //we work with submap bounds if we have them, else we work with the whole map
        let mut submaps : Vec<Rect> = Vec::new();
        if let Some(submaps_builder) = &build_data.submaps {
            submaps = submaps_builder.clone();
        }

        let mut sx = 1;
        let mut sy = 1;
        let mut endx = build_data.map.width-1;
        let mut endy = build_data.map.height-1;

        if submaps.len() > 0{
            sx = submaps[0].x1;
            sy = submaps[0].y1;
            endx = submaps[0].x2;
            endy = submaps[0].y2;

        }

        //fill with floors
        for y in sy .. endy {
            for x in sx .. endx {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::Floor;
            }
        }
        build_data.take_snapshot();


        //place walls around
        //Rust is weird, ranges are inclusive at the beginning but exclusive at the end
        // for x in 0 ..build_data.map.width{
        //     let mut idx = build_data.map.xy_idx(x, 1);
        //     build_data.map.tiles[idx] = TileType::Wall;
        //     idx = build_data.map.xy_idx(x, build_data.map.height-2);
        //     build_data.map.tiles[idx] = TileType::Wall;
        // }
        // for y in 0 ..build_data.map.height{
        //     let mut idx = build_data.map.xy_idx(1, y);
        //     build_data.map.tiles[idx] = TileType::Wall;
        //     idx = build_data.map.xy_idx(build_data.map.width-2, y);
        //     build_data.map.tiles[idx] = TileType::Wall;
        // }

        // build_data.take_snapshot();

        //BSP now
        self.rects.clear();
        self.rects.push( Rect::new(sx, sy, endx-1, endy-1) ); // Start with a single map-sized rectangle
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
                        build_data.map.tiles[idx] = TileType::FloorIndoor;
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

        console::log(format!("Buildings: {:?}", rooms_copy));
        let building_size = self.sort_buildings(&rooms_copy);
        console::log(format!("Buildings sorted: {:?}", building_size));
        self.building_factory(rng, build_data, &rooms_copy, &building_size);
    }

    fn sort_buildings(&mut self, buildings: &Vec<Rect>) -> Vec<(usize, i32, BuildingTag)> 
    {
        let mut building_size : Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i,building) in buildings.iter().enumerate() {
            let rect_width = i32::abs(building.x1 - building.x2);
            let rect_height = i32::abs(building.y1 - building.y2);
            building_size.push((
                i,
                rect_height * rect_width,
                BuildingTag::Unassigned
            ));
        }
        building_size.sort_by(|a,b| b.1.cmp(&a.1));
        building_size[0].2 = BuildingTag::Pub;
        for b in building_size.iter_mut().skip(1) {
            b.2 = BuildingTag::Hovel;
        }

        building_size
    }

    fn building_factory(&mut self, 
        rng: &mut rltk::RandomNumberGenerator, 
        build_data : &mut BuilderMap, 
        buildings: &Vec<Rect>, 
        building_index : &[(usize, i32, BuildingTag)]) 
    {
        for (i,building) in buildings.iter().enumerate() {
            let build_type = &building_index[i].2;
            match build_type {
                BuildingTag::Pub => self.build_pub(&building, build_data, rng),
                _ => {}
            }
        }
    }

    fn build_pub(&mut self, 
        building: &Rect, 
        build_data : &mut BuilderMap, 
        rng: &mut rltk::RandomNumberGenerator) 
    {
        // Place the player
        let cent = building.center();
        build_data.starting_position = Some(Position{
            x : cent.0,
            y : cent.1
        });
        let player_idx = build_data.map.xy_idx(cent.0, cent.1);
    
        // Place other items
        let mut to_place : Vec<&str> = vec!["Barkeep", "Shady Salesman", "Patron", "Patron",
            "Table", "Chair", "Table", "Chair"];
        for y in building.y1 .. building.y2 {
            for x in building.x1 .. building.x2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::FloorIndoor && idx != player_idx && rng.roll_dice(1, 3)==1 && !to_place.is_empty() {
                    let entity_tag = to_place[0];
                    to_place.remove(0);
                    build_data.list_spawns.push((idx, entity_tag.to_string()));
                }
            }
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
                //console::log(&format!("Candidate {:?} overlaps a room {:?}", rect, r));
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
                        //console::log(&format!("Candidate {:?} failed the tile check!", rect));
                        can_build = false; 
                    }
                }
            }
        }

        can_build
    }
}