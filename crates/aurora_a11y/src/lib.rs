//! Accessibility support for Aurora UI via [AccessKit](https://github.com/AccessKit/accesskit).
//!
//! This crate provides [`NodeInfo`], a builder-style struct that widgets use to
//! declare their semantic role, name, value, and state. The platform layer
//! converts these into an AccessKit [`TreeUpdate`](accesskit::TreeUpdate) and
//! pushes it to the native accessibility API each frame.

pub use accesskit;

/// Describes the accessibility semantics of a single widget.
///
/// Widgets return a `NodeInfo` from their `access_info()` method. The platform
/// layer walks the widget tree, collects these, and converts them into AccessKit
/// nodes with correct parent-child relationships and bounds.
///
/// # Example
///
/// ```ignore
/// fn access_info(&self) -> NodeInfo {
///     NodeInfo::new(accesskit::Role::Button)
///         .with_label("Submit")
/// }
/// ```
pub struct NodeInfo {
    pub role: accesskit::Role,
    pub label: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub toggled: Option<accesskit::Toggled>,
    pub numeric_value: Option<f64>,
    pub min_numeric_value: Option<f64>,
    pub max_numeric_value: Option<f64>,
    pub numeric_value_step: Option<f64>,
    pub live: Option<accesskit::Live>,
    pub modal: bool,
    pub expanded: Option<bool>,
    pub hidden: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub multiline: bool,
    pub selected: bool,
    /// If true, the tree walker will skip this widget and promote its children
    /// directly into the parent's child list. Useful for layout wrappers that
    /// have no semantic meaning (e.g. Row, Column, Stack).
    pub transparent: bool,
}

impl Default for NodeInfo {
    fn default() -> Self {
        Self {
            role: accesskit::Role::GenericContainer,
            label: None,
            value: None,
            description: None,
            toggled: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            numeric_value_step: None,
            live: None,
            modal: false,
            expanded: None,
            hidden: false,
            disabled: false,
            read_only: false,
            multiline: false,
            selected: false,
            transparent: false,
        }
    }
}

impl NodeInfo {
    /// Create a new `NodeInfo` with the given role.
    pub fn new(role: accesskit::Role) -> Self {
        Self {
            role,
            ..Default::default()
        }
    }

    /// Create a semantically transparent node whose children are promoted to
    /// the parent. Used for layout wrappers like Row, Column, and Stack.
    pub fn transparent() -> Self {
        Self {
            transparent: true,
            ..Default::default()
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_toggled(mut self, toggled: accesskit::Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.toggled = Some(if checked {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        });
        self
    }

    pub fn with_numeric_value(mut self, value: f64) -> Self {
        self.numeric_value = Some(value);
        self
    }

    pub fn with_numeric_range(mut self, min: f64, max: f64) -> Self {
        self.min_numeric_value = Some(min);
        self.max_numeric_value = Some(max);
        self
    }

    pub fn with_numeric_step(mut self, step: f64) -> Self {
        self.numeric_value_step = Some(step);
        self
    }

    pub fn with_live(mut self, live: accesskit::Live) -> Self {
        self.live = Some(live);
        self
    }

    pub fn with_modal(mut self) -> Self {
        self.modal = true;
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn with_hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn with_disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn with_read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn with_multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn with_selected(mut self) -> Self {
        self.selected = true;
        self
    }

    /// Build an [`accesskit::Node`] from this info, attaching the given child IDs and bounds.
    pub fn build_node(
        &self,
        children: Vec<accesskit::NodeId>,
        bounds: accesskit::Rect,
    ) -> accesskit::Node {
        let mut node = accesskit::Node::new(self.role);
        node.set_bounds(bounds);
        node.set_children(children);

        if let Some(ref label) = self.label {
            node.set_label(label.clone());
        }
        if let Some(ref value) = self.value {
            node.set_value(value.clone());
        }
        if let Some(ref desc) = self.description {
            node.set_description(desc.clone());
        }
        if let Some(toggled) = self.toggled {
            node.set_toggled(toggled);
        }
        if let Some(v) = self.numeric_value {
            node.set_numeric_value(v);
        }
        if let Some(v) = self.min_numeric_value {
            node.set_min_numeric_value(v);
        }
        if let Some(v) = self.max_numeric_value {
            node.set_max_numeric_value(v);
        }
        if let Some(v) = self.numeric_value_step {
            node.set_numeric_value_step(v);
        }
        if let Some(live) = self.live {
            node.set_live(live);
        }
        if self.modal {
            node.set_modal();
        }
        if let Some(expanded) = self.expanded {
            node.set_expanded(expanded);
        }
        if self.hidden {
            node.set_hidden();
        }
        if self.disabled {
            node.set_disabled();
        }
        if self.read_only {
            node.set_read_only();
        }
        if self.selected {
            node.set_selected(true);
        }

        node
    }
}
