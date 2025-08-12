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

static SELECTED_BUTTON_COLOR: [f32; 4] = [0.25, 0.6, 1.0, 1.0];

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
pub fn render_uniform_buttons<I, S>(ui: &imgui::Ui, labels: I, padding: [f32; 2]) -> Option<usize>
where
    I: Clone + IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let iter = labels.clone().into_iter();

    // Pass 1: find max width
    let max_width = iter
        .map(|label| ui.calc_text_size(label.as_ref())[0])
        .fold(0.0, f32::max)
        + padding[0] * 2.0;

    let mut clicked_index = None;

    // Pass 2: render
    let style = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.5, 0.5]));
    for (i, label) in labels.into_iter().enumerate() {
        let height = ui.calc_text_size(label.as_ref())[1] + padding[1] * 2.0;
        if ui.button_with_size(label.as_ref(), [max_width, height]) {
            clicked_index = Some(i);
        }
    }
    style.pop();

    clicked_index
}

type ButtonAction<S> = Box<dyn FnMut(&mut S) + 'static>;

// TODO: Not sure if this is actually useful
/// Renders equal-width buttons, and if one is clicked, runs its action on `state`.
/// Returns the clicked index (if any).
pub fn render_uniform_buttons_do<S>(
    ui: &imgui::Ui,
    items: &mut [(&str, ButtonAction<S>)],
    state: &mut S,
    padding: [f32; 2],
) -> Option<usize> {
    // Pass 1: max width
    let max_width = items
        .iter()
        .map(|(label, _)| ui.calc_text_size(label)[0])
        .fold(0.0, f32::max)
        + padding[0] * 2.0;

    // Pass 2: render
    let mut clicked = None;
    for (i, (label, _)) in items.iter_mut().enumerate() {
        let _style = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.5, 0.5]));
        let height = ui.calc_text_size(&*label)[1] + padding[1] * 2.0;
        if ui.button_with_size(label, [max_width, height]) {
            clicked = Some(i);
        }
    }

    // Dispatch after the render loop to avoid borrow tangles
    if let Some(i) = clicked {
        (items[i].1)(state);
    }

    clicked
}

#[macro_export]
macro_rules! buttons {
    ($($label:expr => $action:expr),+ $(,)?) => {
        vec![$(($label, Box::new($action) as _)),+]
    };
}

pub fn render_button_disabled_conditionally(
    ui: &imgui::Ui,
    label: &str,
    size: [f32; 2],
    condition: bool,
    tooltip: &str,
) -> bool {
    let disabled_token = ui.begin_disabled(condition);

    let clicked = ui.button_with_size(label, size);

    disabled_token.end();

    if ui.is_item_hovered_with_flags(imgui::HoveredFlags::ALLOW_WHEN_DISABLED)
        && condition
        && !tooltip.is_empty()
    {
        ui.tooltip_text(tooltip);
    }

    clicked
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
