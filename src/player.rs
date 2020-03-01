use super::{
    CombatStats, GameLog, Item, Map, Player, Position, RunState, State, TileType, Viewshed,
    WantsToMelee, WantsToPickupItem, Monster,
};
use rltk::{console, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let players = world.read_storage::<Player>();
    let mut positions = world.write_storage::<Position>();
    let mut viewsheds = world.write_storage::<Viewshed>();
    let combat_stats = world.read_storage::<CombatStats>();
    let map = world.fetch::<Map>();
    let mut wants_to_melee = world.write_storage::<WantsToMelee>();
    let entities = world.entities();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        let destination_x = pos.x + delta_x;
        let destination_y = pos.y + delta_y;
        if !map.is_inside_map(destination_x, destination_y) {
            return;
        }

        let destination_idx = map.xy_idx(destination_x, destination_y);
        for potential_target in &map.tile_content[destination_idx] {
            let target = combat_stats.get(*potential_target);
            if target.is_some() {
                console::log(&format!("wanting to melee"));
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("add target failed");
                return;
            }
        }

        if !map.blocked_tiles[destination_idx] {
            pos.x = destination_x;
            pos.y = destination_y;
            viewshed.dirty = true;

            let mut player_position = world.write_resource::<Point>();
            player_position.x = pos.x;
            player_position.y = pos.y;
        }
    }
}

fn get_item(world: &mut World) {
    let player_pos = world.fetch::<Point>();
    let player_entity = world.fetch::<Entity>();
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    let mut gamelog = world.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
            break;
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push(String::from("There is nothing here to pick up.")),
        Some(item) => {
            let mut pickup = world.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.world),
            VirtualKeyCode::Right | VirtualKeyCode::D => try_move_player(1, 0, &mut gs.world),
            VirtualKeyCode::Up | VirtualKeyCode::W => try_move_player(0, -1, &mut gs.world),
            VirtualKeyCode::Down | VirtualKeyCode::S => try_move_player(0, 1, &mut gs.world),
            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.world),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.world),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, &mut gs.world),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.world),
            // Skip Turns
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(&mut gs.world),
            // Interactions
            VirtualKeyCode::G => get_item(&mut gs.world),
            VirtualKeyCode::E => return RunState::ShowInventory,
            VirtualKeyCode::Q => return RunState::ShowDropItem,
            VirtualKeyCode::Period => return try_next_level(&mut gs.world),
            // Save and Quit
            VirtualKeyCode::Escape => return RunState::SaveGame,
            _ => return RunState::AwaitingInput,
        },
    }
    return RunState::PlayerTurn;
}

pub fn try_next_level(world: &mut World) -> RunState {
    let player_pos = world.fetch::<Point>();
    let map = world.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        RunState::NextLevel
    } else {
        let mut gamelog = world.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        RunState::AwaitingInput
    }
}

fn skip_turn(world: &mut World) -> RunState {
    let player_entity = world.fetch::<Entity>();
    let viewshed_components = world.read_storage::<Viewshed>();
    let monsters = world.read_storage::<Monster>();

    let map = world.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = map.xy_idx(tile.x, tile.y);
        for entity_id in map.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false; }
            }
        }
    }

    if can_heal {
        let mut health_components = world.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}
