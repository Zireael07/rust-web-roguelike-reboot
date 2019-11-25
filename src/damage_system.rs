extern crate specs;
use specs::prelude::*;
use super::{Pools, SufferDamage, Player, Name, gamelog::GameLog, RunState};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, Pools>,
                        WriteStorage<'a, SufferDamage> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut pools, mut damage) = data;

        for (mut pool, damage) in (&mut pools, &damage).join() {
            pool.hit_points.current -= damage.amount;
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, pools) in (&entities, &pools).join() {
            if pools.hit_points.current < 1 { 
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", &victim_name.name));
                        }
                        dead.push(entity);
                    },
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                        //log.entries.push("You are dead".to_string())
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}