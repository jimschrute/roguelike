use crate::{CombatStats, GameLog, InBackpack, Map, Name, Player, Position, State};
use rltk::{Console, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

pub fn draw_ui(world: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    ctx.print_color(
        71,
        45,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        &format!("FPS: {}", ctx.fps),
    );
    ctx.print_color(
        61,
        46,
        RGB::named(rltk::CYAN),
        RGB::named(rltk::BLACK),
        &format!("Frame Time: {} ms", ctx.frame_time_ms),
    );

    let combat_stats = world.read_storage::<CombatStats>();
    let players = world.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );

        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );

        let log = world.fetch::<GameLog>();
        let mut y = 44;
        for s in log.entries.iter().rev() {
            if y < 49 {
                ctx.print(2, y, s);
            }
            y += 1;
        }
    }

    draw_tooltips(world, ctx);
}

fn draw_tooltips(world: &World, ctx: &mut Rltk) {
    let map = world.fetch::<Map>();
    let names = world.read_storage::<Name>();
    let positions = world.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }

    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    let idx = map.xy_idx(mouse_pos.0, mouse_pos.1);
    ctx.print(
        58,
        47,
        &format!("idx: {}, pos: {},{}", idx, mouse_pos.0, mouse_pos.1),
    );

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

pub fn show_inventory(game_state: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    show_backpack_menu(game_state, ctx, "Inventory")
}

pub fn show_drop_menu(game_state: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    show_backpack_menu(game_state, ctx, "Drop which Item?")
}

pub fn show_backpack_menu(game_state: &mut State, ctx: &mut Rltk, title: &str) -> ItemMenuResult {
    let player_entity = game_state.world.fetch::<Entity>();
    let names = game_state.world.read_storage::<Name>();
    let backpack = game_state.world.read_storage::<InBackpack>();
    let entities = game_state.world.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        title,
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let inventory = (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity);
    for (i, (entity, _pack, name)) in inventory.enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            97 + i as u8,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    let entity = equippable[selection as usize];
                    return ItemMenuResult::Selected(entity);
                }
                ItemMenuResult::NoResponse
            }
        },
    }
}

pub fn show_path(map: &Map, player_position: &Point, ctx: &mut Rltk) {
    use rltk::Algorithm2D;
    // Either render the proposed path or run along it
    // if game_state. == RunState::Waiting {
    // Render a mouse cursor
    let mouse_pos = ctx.mouse_pos();
    if !map.is_inside_map(mouse_pos.0, mouse_pos.1) {
        return;
    }

    let mouse_idx = map.point2d_to_index(Point::new(mouse_pos.0, mouse_pos.1));
    let player_idx = map.point2d_to_index(*player_position);
    let map_pos = map.pos_from_idx(mouse_idx);

    ctx.print_color(
        mouse_pos.0,
        mouse_pos.1,
        RGB::from_f32(0.0, 1.0, 1.0),
        RGB::from_f32(0.0, 1.0, 1.0),
        "X",
    );

    if map.is_floor_available(map_pos.x, map_pos.y) {
        println!(
            "calculating star search start {} end {}",
            player_idx, mouse_idx
        );
        let path = rltk::a_star_search(player_idx, mouse_idx, map);
        println!("path success {}\n >> steps {:?}", path.success, path.steps);
        if path.success {
            for step in path.steps.iter().skip(1) {
                let step_pos = map.pos_from_idx(*step);
                ctx.print_color(
                    step_pos.x,
                    step_pos.y,
                    RGB::from_f32(1., 0., 0.),
                    RGB::from_f32(0., 0., 0.),
                    "*",
                );
            }

            // if ctx.left_click {
            //     self.mode = Mode::Moving;
            //     self.path = path.clone();
            // }
        }
    }
    // }

    // else {
    //     self.player_position = self.path.steps[0] as usize;
    //     self.path.steps.remove(0);
    //     if self.path.steps.is_empty() {
    //         self.mode = Mode::Waiting;
    //     }
    // }
}
