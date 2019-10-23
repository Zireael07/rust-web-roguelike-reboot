extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};
extern crate rltk;
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {                
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                //&* deref-ref dance to get the map from ECS
                //the FOV seems to be a variant of the shadowcasting algo
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);

                // If this is the player, reveal what they can see
                let _p : Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    //clear map visible tiles
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    // mark as revealed & visible
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}