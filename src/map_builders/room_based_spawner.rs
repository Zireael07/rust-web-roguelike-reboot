use super::{MetaMapBuilder, BuilderMap, spawner};
use rltk::RandomNumberGenerator;

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            //we skip room 1 because we don't want any in starting room
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(&build_data.map, rng, room, &mut build_data.list_spawns);
            }
        } else {
            panic!("Room Based Spawning only works after rooms have been created");
        }
    }
}