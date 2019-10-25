extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Position, Map, Monster, Name};
extern crate rltk;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{field_of_view, Point, console};

pub struct NPCAI {}

impl<'a> System<'a> for NPCAI {
    type SystemData = ( ReadStorage<'a, Viewshed>, 
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>);

    fn run(&mut self, data : Self::SystemData) {
        let (viewshed, pos, monster, name) = data;

        for (viewshed,pos,_monster, name) in (&viewshed, &pos, &monster, &name).join() {
            console::log(&format!("{} considers their own existence", name.name));
        }
    }
}