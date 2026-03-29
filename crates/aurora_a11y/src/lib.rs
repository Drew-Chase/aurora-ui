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

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::Role;

    #[test]
    fn new_sets_role_and_defaults() {
        let info = NodeInfo::new(Role::Button);
        assert_eq!(info.role, Role::Button);
        assert!(info.label.is_none());
        assert!(info.value.is_none());
        assert!(info.description.is_none());
        assert!(info.toggled.is_none());
        assert!(info.numeric_value.is_none());
        assert!(info.expanded.is_none());
        assert!(!info.modal);
        assert!(!info.hidden);
        assert!(!info.disabled);
        assert!(!info.read_only);
        assert!(!info.multiline);
        assert!(!info.selected);
        assert!(!info.transparent);
    }

    #[test]
    fn transparent_node() {
        let info = NodeInfo::transparent();
        assert!(info.transparent);
        assert_eq!(info.role, Role::GenericContainer);
    }

    #[test]
    fn with_label() {
        let info = NodeInfo::new(Role::Button).with_label("Submit");
        assert_eq!(info.label.as_deref(), Some("Submit"));
    }

    #[test]
    fn with_value() {
        let info = NodeInfo::new(Role::TextInput).with_value("hello");
        assert_eq!(info.value.as_deref(), Some("hello"));
    }

    #[test]
    fn with_description() {
        let info = NodeInfo::new(Role::Image).with_description("A photo");
        assert_eq!(info.description.as_deref(), Some("A photo"));
    }

    #[test]
    fn with_checked() {
        let checked = NodeInfo::new(Role::CheckBox).with_checked(true);
        assert_eq!(checked.toggled, Some(accesskit::Toggled::True));

        let unchecked = NodeInfo::new(Role::CheckBox).with_checked(false);
        assert_eq!(unchecked.toggled, Some(accesskit::Toggled::False));
    }

    #[test]
    fn with_numeric_value_and_range() {
        let info = NodeInfo::new(Role::Slider)
            .with_numeric_value(50.0)
            .with_numeric_range(0.0, 100.0)
            .with_numeric_step(1.0);
        assert_eq!(info.numeric_value, Some(50.0));
        assert_eq!(info.min_numeric_value, Some(0.0));
        assert_eq!(info.max_numeric_value, Some(100.0));
        assert_eq!(info.numeric_value_step, Some(1.0));
    }

    #[test]
    fn boolean_flags() {
        let info = NodeInfo::new(Role::Dialog)
            .with_modal()
            .with_hidden()
            .with_disabled()
            .with_read_only()
            .with_multiline()
            .with_selected();
        assert!(info.modal);
        assert!(info.hidden);
        assert!(info.disabled);
        assert!(info.read_only);
        assert!(info.multiline);
        assert!(info.selected);
    }

    #[test]
    fn with_expanded() {
        let info = NodeInfo::new(Role::TreeItem).with_expanded(true);
        assert_eq!(info.expanded, Some(true));
    }

    #[test]
    fn with_live() {
        let info = NodeInfo::new(Role::Alert).with_live(accesskit::Live::Assertive);
        assert_eq!(info.live, Some(accesskit::Live::Assertive));
    }

    #[test]
    fn build_node_basic() {
        let info = NodeInfo::new(Role::Button).with_label("OK");
        let bounds = accesskit::Rect::new(0.0, 0.0, 100.0, 40.0);
        let node = info.build_node(vec![], bounds);
        assert_eq!(node.role(), Role::Button);
    }

    #[test]
    fn build_node_with_children() {
        let info = NodeInfo::new(Role::List);
        let child1 = accesskit::NodeId(1);
        let child2 = accesskit::NodeId(2);
        let bounds = accesskit::Rect::new(0.0, 0.0, 200.0, 300.0);
        let node = info.build_node(vec![child1, child2], bounds);
        assert_eq!(node.role(), Role::List);
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn builder_chain() {
        let info = NodeInfo::new(Role::Button)
            .with_label("Submit")
            .with_description("Submit the form")
            .with_disabled();
        assert_eq!(info.label.as_deref(), Some("Submit"));
        assert_eq!(info.description.as_deref(), Some("Submit the form"));
        assert!(info.disabled);
    }
}
