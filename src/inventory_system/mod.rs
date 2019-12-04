extern crate specs;
use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog, Map,
    WantsToUseItem, MedItem, Pools, WantsToDropItem, Consumable, InflictsDamage, SufferDamage, AreaOfEffect, Confusion, ProvidesFood, ProvidesQuench,
    Equippable, Equipped, EquipmentChanged, WantsToRemoveItem, particle_system};


mod collection_system;
pub use collection_system::ItemCollectionSystem;
mod use_system;
pub use use_system::ItemUseSystem;
mod drop_system;
pub use drop_system::ItemDropSystem;
mod remove_system;
pub use remove_system::ItemRemoveSystem;