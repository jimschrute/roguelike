use crate::{CombatStats, GameLog, Name, SufferDamage, WantsToMelee};
use specs::prelude::*;

pub struct MeleeCombat {}

impl<'a> System<'a> for MeleeCombat {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, combat_stats, mut inflict_damage) = data;

        for (_entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        inflict_damage
                            .insert(wants_melee.target, SufferDamage { amount: damage })
                            .expect("Unable to do damage");
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
