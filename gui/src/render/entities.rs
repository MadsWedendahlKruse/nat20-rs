use hecs::{Entity, World};
use nat20_rs::{
    components::{
        ability::AbilityScoreMap,
        damage::DamageResistances,
        effects::effects::{Effect, EffectDuration},
        hit_points::HitPoints,
        id::{FeatId, Name},
        level::{ChallengeRating, CharacterLevels},
        resource::ResourceMap,
        skill::SkillSet,
        spells::spellbook::Spellbook,
    },
    entities::{character::CharacterTag, monster::MonsterTag},
    systems,
};

use crate::{
    render::{
        components::render_race,
        inventory::render_loadout_inventory,
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
                todo!()
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

impl ImguiRenderableMutWithContext<(&mut World, Entity)> for CharacterTag {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: (&mut World, Entity)) {
        let (world, entity) = context;
        ui.text(format!("ID: {:?}", entity));

        render_race(ui, world, entity);
        systems::helpers::get_component::<CharacterLevels>(world, entity).render(ui);
        systems::helpers::get_component::<HitPoints>(world, entity).render(ui);
        systems::helpers::get_component::<AbilityScoreMap>(world, entity)
            .render_with_context(ui, (world, entity));
        systems::helpers::get_component::<DamageResistances>(world, entity).render(ui);

        if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs{:?}", entity)) {
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
                systems::helpers::get_component::<ResourceMap>(world, entity).render(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Effects") {
                systems::effects::effects(world, entity).render(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Feats") {
                systems::helpers::get_component::<Vec<FeatId>>(world, entity).render(ui);
                tab.end();
            }

            tab_bar.end();
        }
    }
}
