extern crate specs;
use specs::prelude::*;
use crate::{MyTurn, WantsToApproach, Position, Map, Viewshed, EntityMoved};

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut turns, mut want_approach, mut positions, mut map, 
            mut viewsheds, mut entity_moved, entities) = data;
            
        let mut turn_done : Vec<Entity> = Vec::new();
        for (entity, mut pos, approach, mut viewshed, _myturn) in 
            (&entities, &mut positions, &want_approach, &mut viewsheds, &turns).join() 
        {
            turn_done.push(entity);
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32, 
                map.xy_idx(approach.idx % map.width, approach.idx / map.width) as i32, 
                &mut *map
            );
            if path.success && path.steps.len()>1 {
                let mut idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
                pos.x = path.steps[1] % map.width;
                pos.y = path.steps[1] / map.width;
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = true;
                viewshed.dirty = true;
            }
        }

        want_approach.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}