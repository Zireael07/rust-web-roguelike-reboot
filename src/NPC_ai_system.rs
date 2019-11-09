extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Position, Map, Monster, Name};
extern crate rltk;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{field_of_view, Point, console};

pub struct NPCAI {}

impl<'a> System<'a> for NPCAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed,_monster,name,mut pos) in (&mut viewshed, &monster, &name, &mut position).join() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                // Attack goes here
                console::log(&format!("{} shouts insults!", name.name));
                return
            }
            else if viewshed.visible_tiles.contains(&*player_pos){
                //A*
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32, 
                    map.xy_idx(player_pos.x, player_pos.y) as i32, 
                    &mut *map
                );
                //step 0 is always the current location
                if path.success && path.steps.len()>1 {
                    pos.x = path.steps[1] % map.width;
                    pos.y = path.steps[1] / map.width;
                    viewshed.dirty = true;
                }
            }
        }

        // for (player_pos, viewshed,_monster, name) in (&viewshed, &pos, &monster, &name).join() {
        //     console::log(&format!("{} considers their own existence", name.name));
        // }
    }
}