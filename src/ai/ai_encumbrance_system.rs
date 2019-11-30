extern crate specs;
use specs::prelude::*;
use crate::{EquipmentChanged, Item, InBackpack, Equipped, Pools, Attributes, gamelog::GameLog};
use std::collections::HashMap;

pub struct EncumbranceSystem {}

impl<'a> System<'a> for EncumbranceSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut equip_dirty, entities, items, backpacks, wielded, 
            mut pools, attributes, player, mut gamelog) = data;

        if equip_dirty.is_empty() { return; }

        // Build the map of who needs updating
        let mut to_update : HashMap<Entity, (f32)> = HashMap::new(); // (weight) //in the future, maybe some more things, hence the parentheses
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(entity, (0.0));
        }

        // Remove all dirty statements
        equip_dirty.clear();

        // Total up equipped items
        for (item, equipped) in (&items, &wielded).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                *totals += item.weight_lbs;
            }
        }

        // Total up carried items
        for (item, carried) in (&items, &backpacks).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                *totals += item.weight_lbs;
            }
        }

        // Apply the data to Pools
        for (entity, (weight)) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.total_weight = *weight;

                if let Some(attr) = attributes.get(*entity) {
                    let carry_capacity_lbs = (attr.strength.base + attr.strength.modifiers) * 15;
                    if pool.total_weight as i32 > carry_capacity_lbs {
                        // Overburdened
                        if *entity == *player {
                            gamelog.entries.push("You are overburdened.".to_string());
                        }
                    }
                }
            }
        }
    }
}