pub trait ImguiRenderable {
    fn render(&self, ui: &imgui::Ui);
}

pub trait ImguiRenderableMut {
    fn render_mut(&mut self, ui: &imgui::Ui);
}

pub trait ImguiRenderableWithContext<C> {
    fn render_with_context(&self, ui: &imgui::Ui, context: C);
}

pub trait ImguiRenderableMutWithContext<C> {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: C);
}

#[macro_export]
macro_rules! table_with_columns {
    ($ui:expr, $table_name:expr, $( $col_name:expr ),+ $(,)? ) => {{
    use imgui::TableColumnSetup;
    use imgui::TableColumnFlags;
    use imgui::Id;
    use imgui::TableFlags;

    fn table_column(name: &str) -> TableColumnSetup<&str> {
        TableColumnSetup {
            name,
            flags: TableColumnFlags::NO_HIDE,
            init_width_or_weight: 0.0,
            user_id: Id::default(),
        }
    }

    let columns = [ $( table_column($col_name) ),+ ];
    $ui.begin_table_header_with_flags(
        $table_name,
        columns,
        TableFlags::SIZING_FIXED_FIT
        | TableFlags::BORDERS
        | TableFlags::ROW_BG
        | TableFlags::NO_HOST_EXTEND_X,
    )
    }};
}

static SELECTED_BUTTON_COLOR: [f32; 4] = [0.6, 0.6, 1.0, 1.0];

pub fn render_button_selectable(
    ui: &imgui::Ui,
    label: String,
    button_size: [f32; 2],
    selected: bool,
) -> bool {
    let style_color = if selected {
        ui.push_style_color(imgui::StyleColor::Button, SELECTED_BUTTON_COLOR)
    } else {
        ui.push_style_color(
            imgui::StyleColor::Button,
            ui.style_color(imgui::StyleColor::Button),
        )
    };

    let clicked = ui.button_with_size(label, button_size);

    style_color.pop();

    clicked
}

/// Renders a vertical list of same-width buttons with centered text.
/// Returns the index of the clicked button, if any.
pub fn render_uniform_buttons(ui: &imgui::Ui, labels: &[&str], padding: [f32; 2]) -> Option<usize> {
    if labels.is_empty() {
        return None;
    }

    // Measure the widest label
    let max_width = labels
        .iter()
        .map(|label| ui.calc_text_size(label)[0])
        .fold(0.0, f32::max)
        + padding[0] * 2.0;

    let mut clicked_index = None;

    for (i, label) in labels.iter().enumerate() {
        // Optional: push style to center-align text (cosmetic if font is monospaced)
        let style = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.5, 0.5]));

        let height = ui.calc_text_size(label)[1] + padding[1] * 2.0;
        if ui.button_with_size(label, [max_width, height]) {
            clicked_index = Some(i);
        }

        style.pop(); // Remove the text align override
    }

    clicked_index
}

pub fn render_button_disabled_conditionally(
    ui: &imgui::Ui,
    label: &str,
    condition: bool,
    tooltip: &str,
) -> bool {
    let style = if condition {
        Some(ui.push_style_var(imgui::StyleVar::Alpha(0.5))) // make it look disabled
    } else {
        None
    };

    let clicked = ui.button(label);

    if let Some(s) = style {
        s.pop();
    }

    if ui.is_item_hovered() && condition {
        ui.tooltip_text(tooltip);
    }

    clicked && !condition // Only return true if clicked and not disabled
}

pub fn render_empty_button(ui: &imgui::Ui, label: &str) {
    let disabled = ui.begin_disabled(true);
    let color = ui.push_style_color(imgui::StyleColor::Button, [0.3, 0.3, 0.3, 1.0]);
    ui.button(label);
    color.pop();
    disabled.end();
}

pub fn render_window_at_cursor<R, F: FnOnce() -> R>(
    ui: &imgui::Ui,
    name: &str,
    always_auto_resize: bool,
    build_fn: F,
) {
    let cursor_pos = ui.io().mouse_pos;
    ui.window(name)
        .position(cursor_pos, imgui::Condition::Once)
        .always_auto_resize(always_auto_resize)
        .build(build_fn);
}
