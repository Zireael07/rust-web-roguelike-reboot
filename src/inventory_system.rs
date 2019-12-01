extern crate specs;
use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog, Map,
    WantsToUseItem, MedItem, Pools, WantsToDropItem, Consumable, InflictsDamage, SufferDamage, AreaOfEffect, Confusion, ProvidesFood, ProvidesQuench,
    Equippable, Equipped, EquipmentChanged, WantsToRemoveItem, particle_system::ParticleBuilder};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack, mut dirty) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            dirty.insert(pickup.collected_by, EquipmentChanged{}).expect("Unable to insert");

            if pickup.collected_by == *player_entity {
                // the tutorial inserts at 0, so the latest is at the top. we do what is more usual, append, so the latest is at bottom
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack, mut dirty) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{x:0, y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropper_pos.x, y : dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            if entity == *player_entity {
                gamelog.entries.insert(0, format!("You drop up the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        WriteStorage<'a, WantsToRemoveItem>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack.insert(to_remove.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>, //necessary for inflicting damage
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, InflictsDamage>,
                        ReadStorage<'a, MedItem>,
                        WriteStorage<'a, Pools>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, Confusion>,
                        ReadStorage<'a, ProvidesFood>,
                        ReadStorage<'a, ProvidesQuench>,
                        //for equipment
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>,
                        //particles
                        WriteExpect<'a, ParticleBuilder>,
                        ReadStorage<'a, Position>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, map, entities, mut wants_use, names, 
            consumables, inflict_damage, meditems, mut pools, mut suffer_damage, aoe, mut confused, provides_food, provides_quench,
            equippable, mut equipped, mut backpack, mut dirty, mut particle_builder, positions) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged{});

            // Targeting
            let mut targets : Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push( *player_entity ); }
                Some(target) => { 
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            // Single target in tile
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            // AoE
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                                //particles in the whole area
                                particle_builder.request(tile_idx.x, tile_idx.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('░'), 200.0);
                            }
                        }
                    }
                }
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // Remove any items the target has in the item's slot
                    let mut to_unequip : Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert backpack entry");
                    }

                    // Wield the item
                    equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }

            // It it is edible, eat it!
            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {}
                Some(_) => {
                    let target = targets[0];
                    let pool = pools.get_mut(target);
                    if let Some(pool) = pool {
                        pool.hunger = pool.hunger + 150;
                        gamelog.entries.push(format!("You eat the {}.", names.get(useitem.item).unwrap().name));
                    }

                    //destroy if consumable
                    let consumable = consumables.get(useitem.item);
                    match consumable {
                        None => {}
                        Some(_) => {
                            entities.delete(useitem.item).expect("Delete failed");
                        }
                    }
                }
            }

            // It it is drinkable, drink it!
            let item_potable = provides_quench.get(useitem.item);
            match item_potable {
                None => {}
                Some(_) => {
                    let target = targets[0];
                    let pool = pools.get_mut(target);
                    if let Some(pool) = pool {
                        pool.thirst = pool.thirst + 250;
                        gamelog.entries.push(format!("You drink the {}.", names.get(useitem.item).unwrap().name));
                    }

                    //destroy if consumable
                    let consumable = consumables.get(useitem.item);
                    match consumable {
                        None => {}
                        Some(_) => {
                            entities.delete(useitem.item).expect("Delete failed");
                        }
                    }
                }
            }

            //if it's a medkit, heal
            let meditem = meditems.get(useitem.item);
            match meditem {
                None => {}
                Some(meditem) => {
                    for target in targets.iter() {
                        let pool = pools.get_mut(*target);
                        if let Some(pool) = pool {
                            pool.hit_points.current = i32::min(pool.hit_points.max, pool.hit_points.current + meditem.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You use the {}, healing {} hp.", names.get(useitem.item).unwrap().name, meditem.heal_amount));
                            }
                            //particles
                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::GREEN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('♥'), 200.0);
                            }

                            //entities.delete(useitem.item).expect("Delete failed");
                            //destroy if consumable
                            let consumable = consumables.get(useitem.item);
                            match consumable {
                                None => {}
                                Some(_) => {
                                    entities.delete(useitem.item).expect("Delete failed");
                                }
                            }
                        }
                    }
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = useitem.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);

                    for mob in map.tile_content[idx].iter() {
                        // only player can use items for now
                        suffer_damage.insert(*mob, SufferDamage{ amount : damage.damage, from_player: true }).expect("Unable to insert");
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You shoot {} at {}, inflicting {} damage.", item_name.name, mob_name.name, damage.damage));
                        }
                        //particles
                        let pos = positions.get(*mob);
                        if let Some(pos) = pos {
                            particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::RED), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                        }

                        //destroy if consumable
                        let consumable = consumables.get(useitem.item);
                        match consumable {
                            None => {}
                            Some(_) => {
                                entities.delete(useitem.item).expect("Delete failed");
                            }
                        }
                    }
                }
            }

            //confusion
            let mut add_confusion = Vec::new();
            //keep borrow checker happy
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns ));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.push(format!("You use {} on {}, confusing them.", item_name.name, mob_name.name));
                            }
                            //particles
                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::PINK), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
                            }

                            //destroy if consumable
                            let consumable = consumables.get(useitem.item);
                            match consumable {
                                None => {}
                                Some(_) => {
                                    entities.delete(useitem.item).expect("Delete failed");
                                }
                            }
                        }
                    }
                }
            }
            //make targets confused!
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion{ turns: mob.1 }).expect("Unable to insert status");
            }

        }

        wants_use.clear();
    }
}