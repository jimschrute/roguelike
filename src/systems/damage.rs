use crate::{CombatStats, GameLog, Name, Player, SufferDamage};
use specs::{Entity, Join, System, World, WorldExt, WriteStorage};

pub struct Damage {}

impl<'a> System<'a> for Damage {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount;
        }

        damage.clear();
    }
}

pub fn delete_the_dead(world: &mut World) {
    for victim in get_dead_entities(&world) {
        world.delete_entity(victim).expect("Unable to delete");
    }
}

fn get_dead_entities(world: &World) -> Vec<Entity> {
    let mut dead: Vec<Entity> = Vec::new();
    let combat_stats = world.read_storage::<CombatStats>();
    let players = world.read_storage::<Player>();
    let names = world.read_storage::<Name>();
    let mut log = world.write_resource::<GameLog>();
    let entities = world.entities();
    for (entity, stats) in (&entities, &combat_stats).join() {
        if stats.hp < 1 {
            let player = players.get(entity);
            match player {
                None => {
                    dead.push(entity);
                    if let Some(dead_name) = names.get(entity) {
                        log.entries.push(format!("{} is dead", &dead_name.name));
                    }
                }
                _ => println!("You are dead."),
            }
        }
    }
    dead
}
