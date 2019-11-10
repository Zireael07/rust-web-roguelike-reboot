extern crate specs;
use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog, 
    WantsToUseMedkit, MedItem, CombatStats};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                // the tutorial inserts at 0, so the latest is at the top. we do what is more usual, append, so the latest is at bottom
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct MedkitUseSystem {}

impl<'a> System<'a> for MedkitUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseMedkit>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, MedItem>,
                        WriteStorage<'a, CombatStats>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_medkit, names, medkits, mut combat_stats) = data;

        for (entity, medkit, stats) in (&entities, &wants_medkit, &mut combat_stats).join() {
            let meditem = medkits.get(medkit.medkit);
            match meditem {
                None => {}
                Some(meditem) => {
                    stats.hp = i32::max(stats.max_hp, stats.hp + meditem.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You use the {}, healing {} hp.", names.get(medkit.medkit).unwrap().name, meditem.heal_amount));
                    }
                    entities.delete(medkit.medkit).expect("Delete failed");
                }
            }
        }

        wants_medkit.clear();
    }
}