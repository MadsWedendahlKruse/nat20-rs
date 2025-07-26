pub trait ImguiRenderable {
    fn render(&self, ui: &imgui::Ui);
}

pub trait ImguiRenderableMut {
    fn render_mut(&mut self, ui: &imgui::Ui);
}

pub trait ImguiRenderableWithContext<C> {
    fn render_with_context(&self, ui: &imgui::Ui, context: &C);
}

pub trait ImguiRenderableMutWithContext<C> {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: &C);
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
        let _style = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.5, 0.5]));

        let height = ui.calc_text_size(label)[1] + padding[1] * 2.0;
        if ui.button_with_size(label, [max_width, height]) {
            clicked_index = Some(i);
        }

        _style.pop(); // Remove the text align override
    }

    clicked_index
}
