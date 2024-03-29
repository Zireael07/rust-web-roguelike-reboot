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
#[derive(Component, Debug, Clone)]
pub struct Quips {
    pub available : Vec<String>
}

#[derive(Component, Debug, Clone)]
pub struct Vendor {
    pub categories : Vec<String>
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
    pub hit_points : Pool,
    pub hunger : i32,
    pub thirst : i32,
    pub total_weight : f32, //to avoid recalculating
    pub money : f32,
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
    pub amount : i32,
    pub from_player: bool //some things treat player special
}

#[derive(Component, Debug)]
pub struct Item {
    pub weight_lbs : f32,
    pub base_value : f32,
}

#[derive(Component, Debug)]
pub struct MedItem {
    pub heal_amount : i32
}

#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner : Entity
}

#[derive(Component, Debug, Clone)]
pub struct EquipmentChanged {}

#[derive(Component, Debug)]
pub struct Consumable {} //in the sense that it's limited use-only

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

#[derive(Component, Debug, Clone)]
pub struct ProvidesFood {}

#[derive(Component, Debug, Clone)]
pub struct ProvidesQuench {}

#[derive(PartialEq, Copy, Clone)]
pub enum EquipmentSlot { Melee, Shield, Head, Torso, Legs, Feet, Hands }

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
    pub defense : f32
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

#[derive(Component, Debug, Clone)]
pub struct Initiative {
    pub current : i32
}

#[derive(Component, Debug, Clone)]
pub struct MyTurn {}

#[derive(Component, Debug, Clone)]
pub struct Faction {
    pub name : String
}

//Rust enum is really an union, so we can do RandomWaypoint...
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Movement { 
    Static, 
    Random,
    RandomWaypoint{ path : Option<Vec<i32>> },
}

#[derive(Component, Debug, Clone)]
pub struct MoveMode {
    pub mode : Movement
}

#[derive(Component, Debug, Clone)]
pub struct Chasing {
    pub target : Entity
}

//components representing intent
#[derive(Component, Debug, Clone)]
pub struct WantsToApproach {
    pub idx : i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToFlee {
    pub indices : Vec<i32>
}

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
