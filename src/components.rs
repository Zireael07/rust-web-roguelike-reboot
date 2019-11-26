use specs::prelude::*;
use rltk::{RGB};

//ECS
#[derive(Component, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order : i32
}

//the name comes from cartography, ie. 'what do I see?'
#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool
}

//AI
#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug, Clone)]
pub struct Bystander {}

#[derive(Component, Debug, Clone)]
pub struct Vendor {}

#[derive(Component, Debug, Clone)]
pub struct Quips {
    pub available : Vec<String>
}

#[derive(Component, Debug)]
pub struct Name {
    pub name : String
}

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32
}

#[derive(Component, Debug, Clone)]
pub struct Pools {
    pub hit_points : Pool
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub base : i32,
    pub modifiers : i32,
    pub bonus : i32
}

#[derive(Component, Debug, Clone)]
pub struct Attributes {
    pub strength : Attribute,
    pub dexterity : Attribute,
    pub constitution : Attribute,
    pub intelligence : Attribute,
    pub wisdom : Attribute,
    pub charisma : Attribute,
}

#[derive(Component, Debug)]
pub struct WantsToMelee {
    pub target : Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount : i32
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct MedItem {
    pub heal_amount : i32
}

#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner : Entity
}

#[derive(Component, Debug)]
pub struct Consumable {}

#[derive(Component, Debug)]
pub struct Ranged {
    pub range : i32
}

#[derive(Component, Debug)]
pub struct InflictsDamage {
    pub damage : i32
}

#[derive(Component, Debug)]
pub struct AreaOfEffect {
    pub radius : i32
}

#[derive(Component, Debug)]
pub struct Confusion {
    pub turns : i32
}

#[derive(PartialEq, Copy, Clone)]
pub enum EquipmentSlot { Melee, Shield }

#[derive(Component, Clone)]
pub struct Equippable {
    pub slot : EquipmentSlot
}

// See wrapper below for serialization
#[derive(Component)]
pub struct Equipped {
    pub owner : Entity,
    pub slot : EquipmentSlot
}

#[derive(Component, Clone)]
pub struct MeleeWeapon {
    // 1 in 1d4
    pub damage_n_dice : i32,
    // 4 in d4
    pub damage_die_type : i32,
    pub damage_bonus : i32,
}

#[derive(Component, Clone)]
pub struct DefenseBonus {
    pub defense : i32
}

#[derive(Component, Debug, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Clone)]
pub struct SingleActivation {}

#[derive(Component, Debug, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Clone)]
pub struct Door { 
    pub open: bool 
}

#[derive(Component, Clone)]
pub struct LightSource {
    pub color : RGB,
    pub range: i32
}

#[derive(Component, Debug, Clone)]
pub struct EntityMoved {}

//components representing intent
#[derive(Component, Debug)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

#[derive(Component, Debug)]
pub struct WantsToUseItem {
    pub item : Entity,
    pub target : Option<rltk::Point>
}

#[derive(Component, Debug)]
pub struct WantsToDropItem {
    pub item : Entity
}

#[derive(Component, Debug)]
pub struct WantsToRemoveItem {
    pub item : Entity
}

//graphical
#[derive(Component, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms : f32
}

#[derive(Component, Debug)]
pub struct Player {}
