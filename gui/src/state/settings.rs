use std::collections::BTreeMap;

use imgui::{InputTextFlags, TreeNodeFlags};

use crate::{
    render::ui::utils::{ImguiRenderableMut, ImguiRenderableMutWithContext},
    state::{self},
};

#[derive(Clone, Debug)]
pub enum Setting {
    Bool(bool),
    I32(i32),
    F32(f32),
    U16(u16),
    // add more as needed (String, Color, Keybind, etc.)
}

impl ImguiRenderableMutWithContext<&str> for Setting {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, label: &str) {
        match self {
            Setting::Bool(v) => ui.checkbox(label, v),
            Setting::I32(v) => ui.input_scalar(label, v).build(),
            Setting::F32(v) => ui.input_scalar(label, v).build(),
            Setting::U16(v) => ui.input_scalar(label, v).build(),
        };
    }
}

type SettingKey = String;

/// Pure view node for rendering; stores child folders + *full keys* of leaves.
/// No references, so no borrow headaches.
#[derive(Default)]
struct ViewNode {
    children: BTreeMap<SettingKey, ViewNode>,
    leaves: Vec<SettingKey>, // keys that terminate here
}

impl ViewNode {
    pub fn new<'a, I>(keys: I) -> Self
    where
        I: Iterator<Item = &'a str>,
    {
        Self::new_filtered(keys, |_| true)
    }

    /// Build a pruned tree containing only keys that match `pred`, plus their ancestor folders.
    fn new_filtered<'a, I, F>(keys: I, mut pred: F) -> Self
    where
        I: Iterator<Item = &'a str>,
        F: FnMut(&str) -> bool,
    {
        let mut root = ViewNode::default();
        for full in keys {
            if !pred(full) {
                continue;
            }
            let mut node = &mut root;
            let mut parts = full.split('.').peekable();
            while let Some(seg) = parts.next() {
                if parts.peek().is_some() {
                    node = node.children.entry(seg.to_string()).or_default();
                } else {
                    node.leaves.push(full.to_string());
                }
            }
        }
        root
    }
}

// root_path is "" for the root; we build child paths like "render/ui/imgui"
fn render_view_tree(
    ui: &imgui::Ui,
    node: &ViewNode,
    settings: &mut BTreeMap<SettingKey, Setting>,
    title: &str,
    root_path: &str,
    open_all: bool, // true when filtering
) {
    let flags = if open_all {
        TreeNodeFlags::DEFAULT_OPEN
    } else {
        TreeNodeFlags::empty()
    };

    if title.is_empty() {
        for (name, child) in &node.children {
            let next = if root_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", root_path, name)
            };
            render_view_tree(ui, child, settings, name, &next, open_all);
        }
        for key in &node.leaves {
            let _id = ui.push_id(key);
            if let Some(s) = settings.get_mut(key) {
                // Optional: highlight leaf label when search matches
                let label = leaf_label(key);
                s.render_mut_with_context(ui, label);
            }
        }
        return;
    }

    let _id = ui.push_id(root_path);
    ui.tree_node_config(title).flags(flags).build(|| {
        for (name, child) in &node.children {
            let next = format!("{}/{}", root_path, name);
            render_view_tree(ui, child, settings, name, &next, open_all);
        }
        for key in &node.leaves {
            let _lid = ui.push_id(key);
            if let Some(s) = settings.get_mut(key) {
                s.render_mut_with_context(ui, leaf_label(key));
            }
        }
    });
}

pub struct GuiSettings {
    settings: BTreeMap<SettingKey, Setting>,
    view_tree: ViewNode,
    search: String,
}

impl GuiSettings {
    pub fn new(settings: BTreeMap<SettingKey, Setting>) -> Self {
        let view_tree = ViewNode::new(settings.keys().map(String::as_str));
        Self {
            settings,
            view_tree,
            search: String::new(),
        }
    }

    pub fn insert(&mut self, key: &str, setting: Setting) {
        self.settings.insert(key.to_string(), setting);
        // Rebuild view tree to include new key
        self.view_tree = ViewNode::new(self.settings.keys().map(String::as_str));
    }

    pub fn get(&self, key: &str) -> Option<&Setting> {
        self.settings.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Setting> {
        self.settings.get_mut(key)
    }

    pub fn get_bool(&mut self, key: &str) -> &mut bool {
        match self.settings.entry(key.to_string()) {
            std::collections::btree_map::Entry::Occupied(o) => match o.into_mut() {
                Setting::Bool(b) => b,
                _ => panic!("Setting {} is not a bool", key),
            },
            _ => panic!("Setting {} not found", key),
        }
    }
}

impl ImguiRenderableMut for GuiSettings {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        // --- Search bar ---
        let width_token = ui.push_item_width(100.0);
        ui.input_text("Search", &mut self.search)
            .flags(InputTextFlags::AUTO_SELECT_ALL)
            .build();
        width_token.end();

        ui.same_line();
        if ui.button("Clear") {
            self.search.clear();
        }

        ui.separator();

        // Case-insensitive matcher: match on full key OR leaf label
        let query = self.search.trim().to_lowercase();
        let filtering = !query.is_empty();
        let matcher = |full: &str| {
            if query.is_empty() {
                return true;
            }
            let leaf = leaf_label(full);
            full.to_lowercase().contains(&query) || leaf.to_lowercase().contains(&query)
        };

        // Build the (possibly filtered) tree for this frame
        let tree = if filtering {
            &ViewNode::new_filtered(self.settings.keys().map(String::as_str), matcher)
        } else {
            // use the cached unfiltered tree
            &self.view_tree
        };

        // Render with tree nodes; when filtering, default-open everything
        render_view_tree(ui, tree, &mut self.settings, "", "", filtering);
    }
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self::new(BTreeMap::from([
            (
                state::parameters::RENDER_IMGUI_DEMO.to_string(),
                Setting::Bool(false),
            ),
            (
                state::parameters::RENDER_NAVIGATION_DEBUG.to_string(),
                Setting::Bool(false),
            ),
            (
                state::parameters::RENDER_CAMERA_DEBUG.to_string(),
                Setting::Bool(false),
            ),
        ]))
    }
}

/// Helper: last segment of a key for a nice leaf label
fn leaf_label(key: &str) -> &str {
    key.rsplit('.').next().unwrap_or(key)
}
