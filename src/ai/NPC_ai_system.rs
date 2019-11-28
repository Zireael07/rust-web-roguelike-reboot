extern crate specs;
use specs::prelude::*;
use crate::{RunState, Viewshed, Position, Map, Monster, Name, WantsToMelee, Confusion, EntityMoved, MyTurn,
     particle_system::ParticleBuilder};
extern crate rltk;
//console is RLTK's wrapper around either println or the web console macro
use rltk::{field_of_view, Point, console};

pub struct NPCAI {}

impl<'a> System<'a> for NPCAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        //ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, Confusion>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, MyTurn>
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, monster, mut position, mut wants_to_melee, 
            mut confused, mut particle_builder, mut entity_moved, turns) = data;

        //do nothing if not our turn
        //if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed,_monster, mut pos, _turn) in (&entities, &mut viewshed, &monster, &mut position, &turns).join() {
            let mut can_act = true;

            //count down confusion turns if applicable
            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;

                //particle
                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::PINK), 
                    rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
            }

            //normal logic
            if can_act {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    // Attack goes here
                    wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity }).expect("Unable to insert attack");
                    // console::log(&format!("{} shouts insults!", name.name));
                    // return
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
                        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                        viewshed.dirty = true;
                    }
                }
            }
        }

        // for (player_pos, viewshed,_monster, name) in (&viewshed, &pos, &monster, &name).join() {
        //     console::log(&format!("{} considers their own existence", name.name));
        // }
    }
}