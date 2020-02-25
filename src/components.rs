use rltk::{Point, RGB};
use specs::{Component, Entity, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub index: u8,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Monster {}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct SufferDamage {
    pub amount: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Item {}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Consumable {}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Confusion {
    pub turns: i32,
}
