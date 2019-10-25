extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Position, Map, Monster, Name};
extern crate rltk;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{field_of_view, Point, console};

pub struct NPCAI {}

impl<'a> System<'a> for NPCAI {
    type SystemData = ( ReadExpect<'a, Point>,
                        ReadStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>);

    fn run(&mut self, data : Self::SystemData) {
        let (player_pos, viewshed, monster, name) = data;

        for (viewshed, _monster, name) in (&viewshed, &monster, &name).join() {
            if viewshed.visible_tiles.contains(&*player_pos){
                console::log(&format!("{} shouts insults!", name.name));
            }
        }

        // for (player_pos, viewshed,_monster, name) in (&viewshed, &pos, &monster, &name).join() {
        //     console::log(&format!("{} considers their own existence", name.name));
        // }
    }
}