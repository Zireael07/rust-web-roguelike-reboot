extern crate specs;
use specs::prelude::*;
use super::{Attributes, Pools, WantsToMelee, Name, SufferDamage, gamelog::GameLog,
MeleeWeapon, EquipmentSlot, DefenseBonus, Equipped, particle_system::ParticleBuilder, Position};
//console is RLTK's wrapper around either println or the web console macro
use rltk::{console};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Attributes>,
                        ReadStorage<'a, Pools>,
                        WriteStorage<'a, SufferDamage>,
                        //bonuses from equipped stuff
                        ReadStorage<'a, MeleeWeapon>,
                        ReadStorage<'a, DefenseBonus>,
                        ReadStorage<'a, Equipped>,
                        WriteExpect<'a, ParticleBuilder>,
                        ReadStorage<'a, Position>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, attributes, pools, mut inflict_damage, 
            melee_weapons, defense_bonuses, equipped, mut particle_builder, positions, mut rng) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_pools) in (&entities, &wants_melee, &names, &attributes, &pools).join() {
            // Are the attacker and defender alive? Only attack if they are
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();

                let natural_roll = rng.roll_dice(1, 100);
                let attribute_hit_bonus = attacker_attributes.strength.bonus;

                let mut weapon_info = MeleeWeapon{
                    damage_n_dice : 1,
                    damage_die_type : 4,
                    damage_bonus : 0
                };

                for (wielded,melee) in (&equipped, &melee_weapons).join() {
                    if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                        weapon_info = melee.clone();
                    }
                }

                // //item bonuses
                let mut offensive_bonus = 0;
                // for (_item_entity, weapon, equipped_by) in (&entities, &melee_weapon, &equipped).join() {
                //     if equipped_by.owner == entity {
                //         offensive_bonus += weapon.power;
                //     }
                // }

                let modified_hit_roll = natural_roll + attribute_hit_bonus + offensive_bonus;

                //d100 roll under
                if modified_hit_roll < 55 { // temporary target
                    // Target hit! Roll weapon's dice
                    let base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    let attr_damage_bonus = attacker_attributes.strength.bonus;
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    //defense item bonus
                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in (&entities, &defense_bonuses, &equipped).join() {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(0, (base_damage + attr_damage_bonus + weapon_damage_bonus) - defensive_bonus);

                     // the tutorial inserts at 0, so the latest is at the top. we do what is more usual, append, so the latest is at bottom
                     if damage == 0 {
                        log.entries.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.push(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                        inflict_damage.insert(wants_melee.target, SufferDamage{ amount: damage }).expect("Unable to do damage");
                    }
                    //particle
                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                }
                else {
                    //Miss
                    log.entries.push(format!("{} attacks {}, but misses!", &name.name, &target_name.name));
                    //particle
                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::CYAN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                }


                   
            }
        }

        wants_melee.clear();
    }
}