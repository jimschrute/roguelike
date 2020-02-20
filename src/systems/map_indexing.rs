use crate::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexing {}

impl<'a> System<'a> for MapIndexing {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.update_blocked_tiles();
        map.clear_content_index();
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            if blockers.get(entity).is_some() {
                map.blocked_tiles[idx] = true;
            }

            map.tile_content[idx].push(entity);
        }
    }
}
