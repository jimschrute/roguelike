use super::{
    CombatStats, GameLog, Item, Map, Player, Position, RunState, State, Viewshed, WantsToMelee,
    WantsToPickupItem,
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
            // Interactions
            VirtualKeyCode::G => get_item(&mut gs.world),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::Q => return RunState::ShowDropItem,
            _ => return RunState::AwaitingInput,
        },
    }
    return RunState::PlayerTurn;
}
