use crate::{Confusion, Map, Monster, Name, Position, RunState, Viewshed, WantsToMelee};
use rltk::{console, Algorithm2D, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        ReadExpect<'a, RunState>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        Entities<'a>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            mut viewshed,
            run_state,
            player_pos,
            player_entity,
            entities,
            monster,
            name,
            mut position,
            mut wants_to_melee,
            mut confusion,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _monster, name, pos) in
            (&entities, &mut viewshed, &monster, &name, &mut position).join()
        {
            let is_confused = confusion.get_mut(entity);
            let can_act = if let Some(is_confused) = is_confused {
                is_confused.turns -= 1;
                console::log(
                    format!("{} is still confused for {} turns",
                    name.name,
                    is_confused.turns,)
                );
                if is_confused.turns < 1 {
                    confusion.remove(entity);
                    true
                } else {
                    false
                }
            } else {
                true
            };

            if can_act && viewshed.visible_tiles.contains(&*player_pos) {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance <= 2.0 {
                    console::log(&format!("{} shouts insults", name.name));
                }

                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("Unable to insert attack");
                    return;
                }

                let start = map.point2d_to_index(Point::new(pos.x, pos.y));
                let end = map.point2d_to_index(Point::new(player_pos.x, player_pos.y));
                let path = rltk::a_star_search(start, end, &*map);

                if path.success && path.steps.len() > 1 {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked_tiles[idx] = false;

                    let new_pos = map.pos_from_idx(path.steps[1]);
                    pos.x = new_pos.x;
                    pos.y = new_pos.y;
                    let new_idx = map.xy_idx(pos.x, pos.y);
                    map.blocked_tiles[new_idx] = true;

                    viewshed.dirty = true;
                }
            }
        }
    }
}
