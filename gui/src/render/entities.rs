use std::ops::Deref;

use hecs::{Entity, World};
use nat20_rs::{
    components::{
        ability::AbilityScoreMap,
        damage::DamageResistances,
        effects::effects::{Effect, EffectDuration},
        hit_points::HitPoints,
        id::{FeatId, Name, RaceId, SubraceId},
        level::{ChallengeRating, CharacterLevels},
        race::{CreatureSize, CreatureType},
        resource::ResourceMap,
        skill::SkillSet,
        spells::spellbook::Spellbook,
    },
    entities::character::CharacterTag,
    systems,
};

use crate::{
    render::{
        inventory::{render_loadout, render_loadout_inventory},
        utils::{
            ImguiRenderable, ImguiRenderableMut, ImguiRenderableMutWithContext,
            ImguiRenderableWithContext,
        },
    },
    table_with_columns,
};

pub enum CreatureRenderMode {
    Full,
    Compact,
}

impl ImguiRenderableWithContext<(&World, CreatureRenderMode)> for Entity {
    fn render_with_context(&self, ui: &imgui::Ui, context: (&World, CreatureRenderMode)) {
        let (world, mode) = context;

        match mode {
            CreatureRenderMode::Full => {
                let entity = *self;
                ui.text(format!("ID: {:?}", entity));

                if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs{:?}", entity)) {
                    if let Some(tab) = ui.tab_item("Overview") {
                        render_race_if_present(ui, world, entity);

                        render_if_present::<CreatureSize>(ui, world, entity);
                        ui.same_line();
                        render_if_present::<CreatureType>(ui, world, entity);

                        // render_if_present::<Name>(ui, world, *self);
                        render_if_present::<CharacterLevels>(ui, world, *self);
                        render_if_present::<ChallengeRating>(ui, world, *self);
                        render_if_present::<HitPoints>(ui, world, *self);
                        ui.separator_with_text("Armor Class");
                        systems::loadout::armor_class(world, entity).render(ui);
                        systems::helpers::get_component::<AbilityScoreMap>(world, entity)
                            .render_with_context(ui, (world, entity));
                        render_if_present::<DamageResistances>(ui, world, entity);

                        tab.end();
                    }

                    if let Some(tab) = ui.tab_item("Effects") {
                        render_if_present::<Vec<Effect>>(ui, world, entity);
                        render_if_present::<Vec<FeatId>>(ui, world, entity);
                        tab.end();
                    }

                    if let Some(tab) = ui.tab_item("Skills") {
                        systems::helpers::get_component::<SkillSet>(world, entity)
                            .render_with_context(ui, (world, entity));
                        tab.end();
                    }

                    if let Some(tab) = ui.tab_item("Inventory") {
                        render_loadout(ui, world, entity);
                        tab.end();
                    }

                    if let Some(tab) = ui.tab_item("Spellbook") {
                        systems::helpers::get_component::<Spellbook>(world, entity).render(ui);
                        tab.end();
                    }

                    if let Some(tab) = ui.tab_item("Resources") {
                        render_if_present::<ResourceMap>(ui, world, entity);
                        tab.end();
                    }

                    tab_bar.end();
                }
            }

            CreatureRenderMode::Compact => {
                render_if_present::<Name>(ui, world, *self);
                render_if_present::<CharacterLevels>(ui, world, *self);
                render_if_present::<ChallengeRating>(ui, world, *self);
                render_if_present::<HitPoints>(ui, world, *self);
                if let Ok(effects) = world.get::<&Vec<Effect>>(*self) {
                    render_effects_compact(ui, &effects);
                }
            }
        }
    }
}

fn render_if_present<T>(ui: &imgui::Ui, world: &World, entity: Entity)
where
    T: hecs::Component + 'static + ImguiRenderable,
{
    if let Ok(component) = world.get::<&T>(entity) {
        component.render(ui);
    }
}

pub fn render_race_if_present(ui: &imgui::Ui, world: &World, entity: Entity) {
    let mut query = world
        .query_one::<(&RaceId, &Option<SubraceId>)>(entity)
        .unwrap();
    if let Some((race, subrace)) = query.get() {
        (race.clone(), subrace.clone()).render(ui);
    }
}

fn render_effects_compact(ui: &imgui::Ui, effects: &[Effect]) {
    let conditions = effects
        .iter()
        .filter(|e| {
            matches!(
                e.duration(),
                EffectDuration::Temporary { .. } | EffectDuration::Conditional
            )
        })
        .collect::<Vec<_>>();
    ui.separator_with_text("Conditions");
    if !conditions.is_empty() {
        if let Some(table) = table_with_columns!(ui, "Conditions", "Condition", "Duration") {
            for effect in conditions {
                ui.table_next_column();
                ui.text(effect.id().to_string());
                ui.table_next_column();
                effect.duration().render(ui);
            }
            table.end();
        }
    } else {
        ui.text("None");
    }
}

impl ImguiRenderableMutWithContext<(&mut World)> for Entity {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, world: &mut World) {
        let entity = *self;
        ui.text(format!("ID: {:?}", entity));

        if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs{:?}", entity)) {
            if let Some(tab) = ui.tab_item("Overview") {
                render_race_if_present(ui, world, entity);

                render_if_present::<CreatureSize>(ui, world, entity);
                ui.same_line();
                render_if_present::<CreatureType>(ui, world, entity);

                // render_if_present::<Name>(ui, world, *self);
                render_if_present::<CharacterLevels>(ui, world, *self);
                render_if_present::<ChallengeRating>(ui, world, *self);
                render_if_present::<HitPoints>(ui, world, *self);
                systems::helpers::get_component::<AbilityScoreMap>(world, entity)
                    .render_with_context(ui, (world, entity));
                render_if_present::<DamageResistances>(ui, world, entity);

                tab.end();
            }

            if let Some(tab) = ui.tab_item("Effects") {
                render_if_present::<Vec<Effect>>(ui, world, entity);
                render_if_present::<Vec<FeatId>>(ui, world, entity);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Skills") {
                systems::helpers::get_component::<SkillSet>(world, entity)
                    .render_with_context(ui, (world, entity));
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Inventory") {
                render_loadout_inventory(ui, world, entity);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Spellbook") {
                systems::helpers::get_component_mut::<Spellbook>(world, entity).render_mut(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Resources") {
                render_if_present::<ResourceMap>(ui, world, entity);
                tab.end();
            }

            tab_bar.end();
        }
    }
}
