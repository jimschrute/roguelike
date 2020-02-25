use crate::{
    AreaOfEffect, CombatStats, Confusion, Consumable, GameLog, InflictsDamage, Map, Name,
    ProvidesHealing, SufferDamage, WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemUsage {}

impl<'a> System<'a> for ItemUsage {
    type SystemData = (
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            mut gamelog,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            damage,
            area_of_effect,
            mut combat_stats,
            mut suffer_damage,
            mut confusion,
        ) = data;

        for (entity, usage) in (&entities, &wants_use).join() {
            let targets = get_targets(&usage, *player_entity, &area_of_effect, &map);

            apply_healing(
                entity,
                &healing,
                &usage,
                &player_entity,
                &names,
                &targets,
                &mut combat_stats,
                &mut gamelog,
            );

            apply_confusion(
                entity,
                &usage,
                &player_entity,
                &names,
                &targets,
                &mut confusion,
                &mut gamelog,
            );

            apply_damage(
                entity,
                &damage,
                &usage,
                &player_entity,
                &names,
                &targets,
                &mut suffer_damage,
                &mut gamelog,
            );

            clear_consumables(&entities, &consumables, usage.item);
        }

        wants_use.clear();
    }
}

fn clear_consumables(entities: &Entities, consumables: &ReadStorage<Consumable>, item: Entity) {
    let consumable = consumables.get(item);
    if consumable.is_some() {
        entities.delete(item).expect("Delete failed");
    }
}

fn get_targets(
    usage: &WantsToUseItem,
    player_entity: Entity,
    area_of_effect: &ReadStorage<AreaOfEffect>,
    map: &Map,
) -> Vec<Entity> {
    let mut targets: Vec<Entity> = Vec::new();
    match usage.target {
        None => {
            targets.push(player_entity);
        }
        Some(target) => {
            let area_effect = area_of_effect.get(usage.item);
            match area_effect {
                None => {
                    let idx = map.xy_idx(target.x, target.y);
                    for mob in map.tile_content[idx].iter() {
                        targets.push(*mob);
                        break; // Non AoE should get the First One
                    }
                }
                Some(area_effect) => {
                    let mut target_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                    target_tiles.retain(|p| map.is_inside_map(p.x, p.y));
                    for tile_idx in target_tiles.iter() {
                        let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                        for mob in map.tile_content[idx].iter() {
                            targets.push(*mob);
                        }
                    }
                }
            }
        }
    }
    targets
}

fn apply_healing(
    entity: Entity,
    healing: &ReadStorage<ProvidesHealing>,
    usage: &WantsToUseItem,
    player_entity: &Entity,
    names: &ReadStorage<Name>,
    targets: &Vec<Entity>,
    combat_stats: &mut WriteStorage<CombatStats>,
    gamelog: &mut GameLog,
) {
    let heal_item = healing.get(usage.item);
    if let Some(heal_item) = heal_item {
        for target in targets.iter() {
            let stats = combat_stats.get_mut(*target);
            if let Some(stats) = stats {
                stats.hp = i32::min(stats.max_hp, stats.hp + heal_item.heal_amount);
                if entity == *player_entity {
                    gamelog.entries.push(format!(
                        "You drink the {}, healing {} hp.",
                        names.get(usage.item).unwrap().name,
                        heal_item.heal_amount,
                    ));
                }
            }
        }
    }
}

fn apply_damage(
    entity: Entity,
    damage: &ReadStorage<InflictsDamage>,
    usage: &WantsToUseItem,
    player_entity: &Entity,
    names: &ReadStorage<Name>,
    targets: &Vec<Entity>,
    suffer_damage: &mut WriteStorage<SufferDamage>,
    gamelog: &mut GameLog,
) {
    let damage_item = damage.get(usage.item);
    if let Some(damage_item) = damage_item {
        for mob in targets.iter() {
            suffer_damage
                .insert(
                    *mob,
                    SufferDamage {
                        amount: damage_item.damage,
                    },
                )
                .expect("Unable to insert damage to mob");
            if entity == *player_entity {
                let mob_name = names.get(*mob).unwrap();
                let item_name = names.get(usage.item).unwrap();
                gamelog.entries.push(format!(
                    "You used the {} on {}, inflicting {} damage.",
                    item_name.name, mob_name.name, damage_item.damage,
                ));
            }
        }
    }
}

fn apply_confusion(
    entity: Entity,
    usage: &WantsToUseItem,
    player_entity: &Entity,
    names: &ReadStorage<Name>,
    targets: &Vec<Entity>,
    confusion: &mut WriteStorage<Confusion>,
    gamelog: &mut GameLog,
) {
    let mut confusions_to_add = Vec::new();
    {
        let item = confusion.get(usage.item);
        if let Some(item) = item {
            for mob in targets.iter() {
                confusions_to_add.push((*mob, Confusion { turns: item.turns }));
                if entity == *player_entity {
                    let mob_name = names.get(*mob).unwrap();
                    let item_name = names.get(usage.item).unwrap();
                    gamelog.entries.push(format!(
                        "You used the {} on {}, causing {} turns of confusion.",
                        item_name.name, mob_name.name, item.turns,
                    ));
                }
            }
        }
    }

    for (mob, new_confusion) in confusions_to_add {
        confusion
            .insert(mob, new_confusion)
            .expect("Unable to apply confusion to mob");
    }
}
