use rltk::{Console, GameState, Point, Rltk};
use specs::prelude::*;
use specs::{Join, World, WorldExt};

mod components;
use components::*;
mod map;
use map::*;
mod player;
use player::*;
mod rect;
use rect::*;
mod gamelog;
use gamelog::*;
mod gui;
mod spawner;
mod systems;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
}

pub struct State {
    pub world: World,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        systems::damage::delete_the_dead(&mut self.world);
        self.process_map(ctx);
        gui::draw_ui(&self.world, ctx);
        self.run_systems_and_update_state(ctx);
    }
}
impl State {
    fn process_map(&mut self, ctx: &mut Rltk) {
        let map = self.world.fetch::<Map>();
        map.draw(ctx);

        let positions = self.world.read_storage::<Position>();
        let renderables = self.world.read_storage::<Renderable>();

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| a.1.index.cmp(&b.1.index));

        for (pos, render) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }

    fn run_systems_and_update_state(&mut self, ctx: &mut Rltk) {
        let run_state = *self.world.fetch::<RunState>();
        let new_state = match run_state {
            RunState::AwaitingInput => player_input(self, ctx),
            RunState::PreRun => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::PlayerTurn => {
                self.run_systems();
                RunState::MonsterTurn
            }
            RunState::MonsterTurn => {
                self.run_systems();
                RunState::AwaitingInput
            }
            RunState::ShowInventory => match gui::show_inventory(self, ctx) {
                gui::ItemMenuResult::Selected(entity) => {
                    let mut intent = self.world.write_storage::<WantsToDrinkPotion>();
                    intent
                        .insert(
                            *self.world.fetch::<Entity>(),
                            WantsToDrinkPotion { potion: entity },
                        )
                        .expect("Unable to insert select inventory intent");
                    RunState::PlayerTurn
                }
                gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                _ => RunState::ShowInventory,
            },
            RunState::ShowDropItem => match gui::show_drop_menu(self, ctx) {
                gui::ItemMenuResult::Selected(entity) => {
                    let mut intent = self.world.write_storage::<WantsToDropItem>();
                    intent
                        .insert(
                            *self.world.fetch::<Entity>(),
                            WantsToDropItem { item: entity },
                        )
                        .expect("Unable to insert drop intent");
                    RunState::PlayerTurn
                }
                gui::ItemMenuResult::Cancel => RunState::AwaitingInput,
                _ => RunState::ShowDropItem,
            },
        };

        let mut run_state_writer = self.world.write_resource::<RunState>();
        *run_state_writer = new_state;
    }

    fn run_systems(&mut self) {
        let mut map_indexing = systems::MapIndexing {};
        map_indexing.run_now(&self.world);
        let mut visibility = systems::Visibility {};
        visibility.run_now(&self.world);
        let mut monster_ai = systems::MonsterAI {};
        monster_ai.run_now(&self.world);
        let mut melee_combat = systems::MeleeCombat {};
        melee_combat.run_now(&self.world);
        let mut damage = systems::Damage {};
        damage.run_now(&self.world);
        let mut pickup = systems::Inventory {};
        pickup.run_now(&self.world);
        let mut potion_usage = systems::PotionUsage {};
        potion_usage.run_now(&self.world);
        let mut item_drop = systems::ItemDrop {};
        item_drop.run_now(&self.world);
        self.world.maintain();
    }
}

fn main() {
    let mut gs = State {
        world: World::new(),
    };
    gs.world.register::<Position>();
    gs.world.register::<Renderable>();
    gs.world.register::<Player>();
    gs.world.register::<Monster>();
    gs.world.register::<Name>();
    gs.world.register::<Viewshed>();
    gs.world.register::<BlocksTile>();
    gs.world.register::<CombatStats>();
    gs.world.register::<WantsToMelee>();
    gs.world.register::<SufferDamage>();
    gs.world.register::<Item>();
    gs.world.register::<Potion>();
    gs.world.register::<WantsToPickupItem>();
    gs.world.register::<InBackpack>();
    gs.world.register::<WantsToDrinkPotion>();
    gs.world.register::<WantsToDropItem>();

    let seed: u64 = 25021990;
    let mut rng = rltk::RandomNumberGenerator::seeded(seed);

    let map = Map::new_rooms_and_corridors(&mut rng);

    let initial_player_pos = map.rooms[0].center();

    gs.world.insert(rng);

    spawner::spawn_map_rooms(&mut gs.world, &map);

    gs.world
        .insert(Point::new(initial_player_pos.x, initial_player_pos.y));
    let player_entity = spawner::player(&mut gs.world, initial_player_pos);

    gs.world.insert(map);
    gs.world.insert(player_entity);
    gs.world.insert(RunState::PreRun);
    gs.world.insert(gamelog::GameLog {
        entries: vec![String::from("Welcome to Rusty Roguelike")],
    });

    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();
    context.with_post_scanlines(true);
    rltk::main_loop(context, gs);
}
