use crate::{Map, Monster, Name, Position, RunState, Viewshed, WantsToMelee};
use rltk::{console, Point};
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
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _monster, name, pos) in
            (&entities, &mut viewshed, &monster, &name, &mut position).join()
        {
            if viewshed.visible_tiles.contains(&*player_pos) {
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

                let start = map.xy_idx(pos.x, pos.y);
                let end = map.xy_idx(player_pos.x, player_pos.y);
                console::log(format!(
                    "calculating star search start {} end {}",
                    start, end
                ));
                let path = rltk::a_star_search(start, end, &mut *map);
                console::log(format!(
                    "monster_pos {:?}, player_pos {:?}, path success {}\n >> steps {:?}",
                    pos, *player_pos, path.success, path.steps
                ));

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
