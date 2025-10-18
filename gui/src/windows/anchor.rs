use std::collections::{BTreeMap, HashMap};

pub static AUTO_RESIZE: [f32; 2] = [0.0, 0.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalAnchor {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowAnchor {
    pub horizontal: HorizontalAnchor,
    pub vertical: VerticalAnchor,
}

impl WindowAnchor {
    pub fn new(horizontal: HorizontalAnchor, vertical: VerticalAnchor) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

pub static TOP_LEFT: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Left,
    vertical: VerticalAnchor::Top,
};

pub static TOP_CENTER: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Center,
    vertical: VerticalAnchor::Top,
};

pub static TOP_RIGHT: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Right,
    vertical: VerticalAnchor::Top,
};

pub static CENTER_LEFT: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Left,
    vertical: VerticalAnchor::Center,
};

pub static CENTER: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Center,
    vertical: VerticalAnchor::Center,
};

pub static CENTER_RIGHT: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Right,
    vertical: VerticalAnchor::Center,
};

pub static BOTTOM_LEFT: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Left,
    vertical: VerticalAnchor::Bottom,
};

pub static BOTTOM_CENTER: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Center,
    vertical: VerticalAnchor::Bottom,
};

pub static BOTTOM_RIGHT: WindowAnchor = WindowAnchor {
    horizontal: HorizontalAnchor::Right,
    vertical: VerticalAnchor::Bottom,
};

#[derive(Debug, Clone)]
pub struct AnchoredWindow {
    pub label: String,
    pub anchor: WindowAnchor,
    pub last_size: [f32; 2],
}

impl AnchoredWindow {
    pub fn new(label: String, anchor: WindowAnchor) -> Self {
        Self {
            label,
            anchor,
            last_size: [0.0, 0.0],
        }
    }

    pub fn update_size(&mut self, size: [f32; 2]) {
        self.last_size = size;
    }
}

#[derive(Debug, Clone)]
pub struct WindowManager {
    pub windows_by_label: BTreeMap<String, AnchoredWindow>,
    pub windos_by_anchor: HashMap<WindowAnchor, Vec<String>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows_by_label: BTreeMap::new(),
            windos_by_anchor: HashMap::new(),
        }
    }

    pub fn add(&mut self, window: AnchoredWindow) {
        let window_label = window.label.clone();
        self.windos_by_anchor
            .entry(window.anchor)
            .or_insert_with(Vec::new)
            .push(window_label);
        self.windows_by_label.insert(window.label.clone(), window);
    }

    fn add_if_missing(&mut self, label: &str, anchor: &WindowAnchor) {
        if !self.windows_by_label.contains_key(label) {
            let window = AnchoredWindow::new(label.to_string(), anchor.clone());
            self.add(window);
        }
    }

    fn window_position(&self, label: &str, ui: &imgui::Ui) -> Option<[f32; 2]> {
        let window = self.windows_by_label.get(label)?;

        let viewport = ui.io().display_size;
        let size = window.last_size;

        // TODO: Take into account that there can be multiple windows with the same anchor
        let x = match window.anchor.horizontal {
            HorizontalAnchor::Left => 0.0,
            HorizontalAnchor::Center => (viewport[0] - size[0]) / 2.0,
            HorizontalAnchor::Right => viewport[0] - size[0],
        };

        let y = match window.anchor.vertical {
            VerticalAnchor::Top => 0.0,
            VerticalAnchor::Center => (viewport[1] - size[1]) / 2.0,
            VerticalAnchor::Bottom => viewport[1] - size[1],
        };

        Some([x, y])
    }

    pub fn render_window<R, F: FnOnce() -> R>(
        &mut self,
        ui: &imgui::Ui,
        label: &str,
        anchor: &WindowAnchor,
        size: [f32; 2],
        opened: &mut bool,
        build_fn: F,
    ) {
        if !*opened {
            return;
        }

        self.add_if_missing(label, anchor);

        let position = self.window_position(label, ui).unwrap_or([0.0, 0.0]);

        let mut actual_size = None;

        ui.window(label)
            .position(position, imgui::Condition::Always)
            .size(size, imgui::Condition::Always)
            .movable(false)
            .opened(opened)
            .build(|| {
                actual_size = Some(ui.window_size());
                build_fn();
            });

        if let Some(window) = self.windows_by_label.get_mut(label)
            && let Some(size) = actual_size
        {
            window.update_size(size);
        }
    }
}
