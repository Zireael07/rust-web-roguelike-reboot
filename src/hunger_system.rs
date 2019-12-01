use specs::prelude::*;
use super::{RunState, Pools, gamelog::GameLog, MyTurn};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        WriteStorage<'a, Pools>,
                        ReadExpect<'a, Entity>, // The player
                        ReadExpect<'a, RunState>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, MyTurn>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut pools, player_entity, runstate, mut log, turns) = data;

        for (entity, mut pool, _myturn) in (&entities, &mut pools, &turns).join() {
            //player only!
            if entity == *player_entity {
                pool.hunger -= 1;
                pool.thirst -= 1;
            }
        }
    }
}