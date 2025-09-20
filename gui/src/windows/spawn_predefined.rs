use hecs::{Entity, World};
use nat20_rs::{
    components::{id::Name, resource::RechargeRule},
    entities::{
        character::{Character, CharacterTag},
        monster::{Monster, MonsterTag},
    },
    systems,
    test_utils::fixtures,
};

use crate::render::{
    entities::CreatureRenderMode,
    utils::{ImguiRenderableMutWithContext, ImguiRenderableWithContext, render_window_at_cursor},
};

pub struct SpawnPredefinedWindow {
    /// Dummy World used to store the predefined entities. Once an entity has been
    /// selected from this window, it will be spawned into the actual game world.
    world: World,
    spawning_completed: bool,
}

impl SpawnPredefinedWindow {
    pub fn new() -> Self {
        let mut world = World::new();

        let spawners = vec![
            fixtures::creatures::heroes::fighter,
            fixtures::creatures::heroes::wizard,
            fixtures::creatures::heroes::warlock,
            fixtures::creatures::monsters::goblin_warrior,
        ];

        for spawner in spawners {
            let entity = spawner(&mut world).id();
            println!("Spawned predefined entity: {:?}", entity);
            // Ensure all resources are fully recharged
            systems::time::pass_time(&mut world, entity, &RechargeRule::LongRest);
        }

        Self {
            world,
            spawning_completed: false,
        }
    }

    pub fn is_spawning_completed(&self) -> bool {
        self.spawning_completed
    }
}

impl ImguiRenderableMutWithContext<&mut World> for SpawnPredefinedWindow {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, main_world: &mut World) {
        if self.spawning_completed {
            return;
        }

        render_window_at_cursor(ui, "Spawn", true, || {
            let mut entity_to_spawn = None;

            self.world
                .query::<&Name>()
                .into_iter()
                .for_each(|(entity, name)| {
                    if ui.collapsing_header(
                        format!("{}##{:?}", name.as_str(), entity),
                        imgui::TreeNodeFlags::FRAMED,
                    ) {
                        if ui.button(format!("Spawn##{:?}", entity)) {
                            println!("Spawning entity: {:?}", entity);
                            entity_to_spawn = Some(entity);
                        }
                        ui.separator();
                        entity.render_with_context(ui, (&self.world, CreatureRenderMode::Full));
                    }
                });

            if let Some(entity) = entity_to_spawn {
                let spawned_entity = if let Ok(_) = self.world.get::<&CharacterTag>(entity) {
                    main_world.spawn(Character::from_world(&self.world, entity))
                } else if let Ok(_) = self.world.get::<&MonsterTag>(entity) {
                    main_world.spawn(Monster::from_world(&self.world, entity))
                } else {
                    panic!("Entity to spawn is neither a Character nor a Monster");
                };

                // Ensure the spawned entity has a unique name in the main world
                // (much easier to debug this way)
                set_unique_name(main_world, spawned_entity);

                entity_to_spawn.take();
            }

            ui.separator();
            if ui.button_with_size("Done", [100.0, 30.0]) {
                self.spawning_completed = true;
            }
        });
    }
}

fn set_unique_name(world: &mut World, entity: Entity) {
    let name = if let Ok(name) = world.get::<&Name>(entity) {
        name.as_str().to_string()
    } else {
        "Unnamed".to_string()
    };
    let mut unique_name = name.clone();
    let mut counter = 0;

    while world
        .query::<&Name>()
        .iter()
        .any(|(_, n)| n.as_str() == unique_name)
    {
        unique_name = format!("{} ({})", name, counter);
        counter += 1;
    }

    if counter < 1 {
        // Name is already unique, no need to update
        return;
    }

    if let Ok(mut name) = world.get::<&mut Name>(entity) {
        *name = Name::new(unique_name);
    }
}
