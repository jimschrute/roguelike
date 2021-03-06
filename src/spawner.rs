use crate::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item, Map,
    Monster, Name, Player, Position, ProvidesHealing, Ranged, Rect, Renderable, SerializeMe,
    TileType, Viewshed,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

const MAX_MONSTERS: i32 = 4;
const MIN_MONSTERS: i32 = 0;
const MAX_ITEMS: i32 = 2;
const MIN_ITEMS: i32 = 0;

/// Spawns the player and returns his/her entity object.
pub fn player(world: &mut World, initial_player_pos: Position) -> Entity {
    world
        .create_entity()
        .with(Player {})
        .with(initial_player_pos)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::from_f32(0.996, 0.906, 0.38),
            bg: RGB::named(rltk::BLACK),
            index: 100,
        })
        .with(Name {
            name: "Player".to_owned(),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

pub fn spawn_map_rooms(world: &mut World, map: &Map) {
    for room in map.rooms.iter().skip(1) {
        spawn_room(world, room, map);
    }
}

/// Fills a room with stuff!
pub fn spawn_room(world: &mut World, room: &Rect, map: &Map) {
    let (monster_spawn_points, item_spawn_points) = {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        (
            generate_monsters_for_room(&mut rng, room, map),
            generate_items_for_room(&mut rng, room, map),
        )
    };

    for idx in monster_spawn_points {
        let pos = map.pos_from_idx(idx);
        random_monster(world, pos);
    }

    for idx in item_spawn_points {
        let pos = map.pos_from_idx(idx);
        random_item(world, pos);
    }
}

fn generate_monsters_for_room(
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map: &Map,
) -> Vec<usize> {
    let mut monsters = Vec::new();

    let num_monsters = rng.range(MIN_MONSTERS, MAX_MONSTERS + 1);

    for _i in 0..num_monsters {
        let mut added = false;
        while !added {
            let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
            let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
            let idx = (y * map.width as usize) + x;
            if !monsters.contains(&idx) && map.tiles[idx] == TileType::Floor {
                monsters.push(idx);
                added = true;
            }
        }
    }

    monsters
}

fn generate_items_for_room(rng: &mut RandomNumberGenerator, room: &Rect, map: &Map) -> Vec<usize> {
    let mut items = Vec::new();

    let num_items = rng.range(MIN_ITEMS, MAX_ITEMS + 1);

    for _i in 0..num_items {
        let mut added = false;
        while !added {
            let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
            let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
            let idx = (y * map.width as usize) + x;
            if !items.contains(&idx) {
                items.push(idx);
                added = true;
            }
        }
    }

    items
}

/// Spawns a random monster at a given location
pub fn random_monster(world: &mut World, pos: Position) {
    let roll: i32;
    {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(world, pos),
        _ => goblin(world, pos),
    }
}

fn orc(ecs: &mut World, pos: Position) {
    monster(ecs, pos, rltk::to_cp437('o'), "Orc");
}
fn goblin(ecs: &mut World, pos: Position) {
    monster(ecs, pos, rltk::to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(world: &mut World, pos: Position, glyph: u8, name: S) -> Entity {
    world
        .create_entity()
        .with(pos)
        .with(Renderable {
            glyph,
            fg: RGB::from_f32(0.894, 0.231, 0.267),
            bg: RGB::named(rltk::BLACK),
            index: 10,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 3,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

pub fn random_item(world: &mut World, pos: Position) {
    let roll: i32;
    {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 4);
    }
    match roll {
        1 => health_potion(world, pos),
        2 => fireball_scroll(world, pos),
        3 => confusion_scroll(world, pos),
        _ => magic_missile_scroll(world, pos),
    }
}

fn health_potion(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            index: 10,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('S'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            index: 10,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('S'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            index: 10,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('S'),
            fg: RGB::named(rltk::IVORY),
            bg: RGB::named(rltk::BLACK),
            index: 10,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(AreaOfEffect { radius: 3 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
