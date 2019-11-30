extern crate specs;
use specs::prelude::*;
use super::{Pools, SufferDamage, Player, Name, gamelog::GameLog, RunState,
Position, Equipped, InBackpack};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, Pools>,
                        WriteStorage<'a, SufferDamage>,
                        ReadExpect<'a, Entity>,
                    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut pools, mut damage, player) = data;
        let mut money_gain = 0.0f32;

        for (mut pool, damage) in (&mut pools, &damage).join() {
            pool.hit_points.current -= damage.amount;

            // if player, gain money
            if pool.hit_points.current < 1 && damage.from_player {
                money_gain += pool.money;
            }
        }

        //effectively auto-pickup money
        if money_gain != 0.0 {
            let mut player_stats = pools.get_mut(*player).unwrap();
            player_stats.money += money_gain;
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


    // Drop stuff
    { // To avoid keeping hold of borrowed entries, use a scope
        let mut to_drop : Vec<(Entity, Position)> = Vec::new();
        let entities = ecs.entities();
        let mut equipped = ecs.write_storage::<Equipped>();
        let mut carried = ecs.write_storage::<InBackpack>();
        let mut positions = ecs.write_storage::<Position>();
        let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
        // Drop everything held by dead people
        for victim in dead.iter() {        
            let pos = positions.get(*victim);
            for (entity, equipped) in (&entities, &equipped).join() {
                if equipped.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }
            for (entity, backpack) in (&entities, &carried).join() {
                if backpack.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }
        }

        for drop in to_drop.iter() {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions.insert(drop.0, drop.1.clone()).expect("Unable to insert position");
        }        
    }


    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}