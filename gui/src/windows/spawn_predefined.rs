use core::f32;

use hecs::{Entity, World};
use imgui::{MouseButton, sys};
use nat20_rs::{
    components::{id::Name, resource::RechargeRule},
    entities::{
        character::{Character, CharacterTag},
        monster::{Monster, MonsterTag},
    },
    systems::{
        self,
        geometry::{CreaturePose, RaycastResult},
    },
    test_utils::fixtures,
};
use parry3d::na::Point3;

use crate::render::ui::{
    entities::CreatureRenderMode,
    utils::{ImguiRenderableMutWithContext, ImguiRenderableWithContext, render_window_at_cursor},
};

pub struct SpawnPredefinedWindow {
    /// Dummy World used to store the predefined entities. Once an entity has been
    /// selected from this window, it will be spawned into the actual game world.
    world: World,
    entity_to_spawn: Option<Entity>,
    current_entity: Option<Entity>,
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
            entity_to_spawn: None,
            current_entity: None,
            spawning_completed: false,
        }
    }

    pub fn is_spawning_completed(&self) -> bool {
        self.spawning_completed
    }
}

impl ImguiRenderableMutWithContext<(&mut World, &mut Option<RaycastResult>)>
    for SpawnPredefinedWindow
{
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        context: (&mut World, &mut Option<RaycastResult>),
    ) {
        if self.spawning_completed {
            return;
        }

        let (main_world, raycast_result) = context;

        render_window_at_cursor(ui, "Spawn", true, || {
            self.world
                .query::<&Name>()
                .into_iter()
                .for_each(|(entity, name)| {
                    if ui.collapsing_header(
                        format!("{}##{:?}", name.as_str(), entity),
                        imgui::TreeNodeFlags::FRAMED,
                    ) {
                        if ui.button(format!("Spawn##{:?}", entity)) {
                            self.entity_to_spawn = Some(entity);
                        }
                        ui.separator();
                        entity.render_with_context(ui, (&self.world, &CreatureRenderMode::Full));
                    }
                });

            if let Some(entity) = self.entity_to_spawn {
                if self.current_entity.is_none() {
                    let spawned_entity = if let Ok(_) = self.world.get::<&CharacterTag>(entity) {
                        main_world.spawn(Character::from_world(&self.world, entity))
                    } else if let Ok(_) = self.world.get::<&MonsterTag>(entity) {
                        main_world.spawn(Monster::from_world(&self.world, entity))
                    } else {
                        panic!("Entity to spawn is neither a Character nor a Monster");
                    };

                    // Spawn it somewhere we can't see it, we'll move it later
                    systems::geometry::move_to(
                        main_world,
                        spawned_entity,
                        Point3::new(f32::MAX, f32::MAX, f32::MAX),
                    );

                    // Ensure the spawned entity has a unique name in the main world
                    // (much easier to debug this way)
                    set_unique_name(main_world, spawned_entity);

                    self.current_entity = Some(spawned_entity);
                }

                if let Some(entity) = self.current_entity {
                    ui.tooltip(|| {
                        ui.text("LEFT-CLICK: Spawn here");
                        ui.text("RIGHT-CLICK: Cancel");
                    });

                    if ui.is_mouse_clicked(MouseButton::Right) {
                        raycast_result.take();
                        main_world.despawn(entity).unwrap();
                        self.current_entity = None;
                        self.entity_to_spawn = None;
                    }

                    if let Some(raycast) = raycast_result {
                        if let Some(raycast_outcome) = raycast.world_hit() {
                            let mut position = raycast_outcome.poi;
                            let creature_height =
                                systems::geometry::get_height(main_world, entity).unwrap();
                            position.y += creature_height / 2.0;

                            systems::geometry::move_to(main_world, entity, position);

                            if ui.is_mouse_clicked(MouseButton::Left) {
                                raycast_result.take();
                                self.current_entity = None;
                            }
                        }
                    }
                }
            }

            ui.separator();
            if ui.button_with_size("Done", [100.0, 30.0]) {
                if let Some(entity) = self.current_entity {
                    main_world.despawn(entity).unwrap();
                    self.current_entity = None;
                    self.entity_to_spawn = None;
                }
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
