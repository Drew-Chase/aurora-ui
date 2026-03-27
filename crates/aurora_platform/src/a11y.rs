//! AccessKit integration for Aurora UI.
//!
//! This module bridges the widget tree with the platform's native accessibility
//! API via [`accesskit_winit`]. After each layout-and-paint cycle the framework
//! walks the widget tree, builds an AccessKit [`TreeUpdate`], and pushes it
//! through the adapter so screen readers see the current UI state.

use aurora_a11y::accesskit::{
    self, Action, ActionRequest, Node, NodeId, Role, Tree, TreeId, TreeUpdate,
};
use aurora_core::geometry::rect::Rect;
use aurora_widgets::widgets::Widget;
use std::sync::{Arc, Mutex};

/// Reserved node ID for the window-level root of the accessibility tree.
const WINDOW_ROOT_ID: u64 = u64::MAX;

/// Manages the AccessKit adapter and action queue.
pub(crate) struct A11yState {
    pub adapter: accesskit_winit::Adapter,
    action_queue: Arc<Mutex<Vec<ActionRequest>>>,
}

// ---------------------------------------------------------------------------
// Handler impls (must be Send + 'static for accesskit_winit)
// ---------------------------------------------------------------------------

struct ActivationHandler {
    title: String,
}

impl accesskit::ActivationHandler for ActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        // Return a minimal tree; the real one is pushed after first layout.
        let mut root = Node::new(Role::Window);
        root.set_label(self.title.clone());
        Some(TreeUpdate {
            nodes: vec![(NodeId(WINDOW_ROOT_ID), root)],
            tree: Some(Tree::new(NodeId(WINDOW_ROOT_ID))),
            tree_id: TreeId::ROOT,
            focus: NodeId(WINDOW_ROOT_ID),
        })
    }
}

struct ActionHandler {
    queue: Arc<Mutex<Vec<ActionRequest>>>,
}

impl accesskit::ActionHandler for ActionHandler {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut q) = self.queue.lock() {
            q.push(request);
        }
    }
}

struct DeactivationHandler;

impl accesskit::DeactivationHandler for DeactivationHandler {
    fn deactivate_accessibility(&mut self) {}
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl A11yState {
    /// Create a new accessibility state and adapter for the given window.
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: &winit::window::Window,
        title: &str,
    ) -> Self {
        let action_queue = Arc::new(Mutex::new(Vec::new()));
        let adapter = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            window,
            ActivationHandler {
                title: title.to_string(),
            },
            ActionHandler {
                queue: action_queue.clone(),
            },
            DeactivationHandler,
        );
        Self {
            adapter,
            action_queue,
        }
    }

    /// Forward a winit window event to the AccessKit adapter.
    pub fn process_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) {
        self.adapter.process_event(window, event);
    }

    /// Rebuild and push the full accessibility tree from the widget tree.
    pub fn update_tree(
        &mut self,
        root_widget: &dyn Widget,
        focused_id: Option<u64>,
        window_rect: Rect,
    ) {
        self.adapter.update_if_active(|| {
            build_tree_update(root_widget, focused_id, window_rect)
        });
    }

    /// Drain buffered action requests from assistive technology.
    ///
    /// Call this before dispatching events so screen-reader-initiated
    /// focus/click requests are processed in the current frame.
    pub fn drain_actions(&self) -> Vec<ActionRequest> {
        self.action_queue
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Tree building
// ---------------------------------------------------------------------------

fn build_tree_update(
    root_widget: &dyn Widget,
    focused_id: Option<u64>,
    window_rect: Rect,
) -> TreeUpdate {
    let mut nodes = Vec::new();
    let mut next_auto_id: u64 = 1;

    // Walk the widget tree to collect accessibility nodes.
    let child_ids = walk_widget(root_widget, window_rect, &mut next_auto_id, &mut nodes);

    // Create the window root node.
    let mut root = Node::new(Role::Window);
    root.set_bounds(to_ak_rect(window_rect));
    root.set_children(child_ids);

    let root_node_id = NodeId(WINDOW_ROOT_ID);
    nodes.push((root_node_id, root));

    let focus = focused_id.map(NodeId).unwrap_or(root_node_id);

    TreeUpdate {
        nodes,
        tree: Some(Tree::new(root_node_id)),
        tree_id: TreeId::ROOT,
        focus,
    }
}

/// Recursively walk the widget tree and emit AccessKit nodes.
///
/// Transparent widgets (layout containers with no semantic role) are
/// skipped — their children are promoted into the parent's child list.
fn walk_widget(
    widget: &dyn Widget,
    bounds: Rect,
    next_id: &mut u64,
    nodes: &mut Vec<(NodeId, Node)>,
) -> Vec<NodeId> {
    let info = widget.access_info();

    if info.transparent {
        // Promote children directly into parent's child list.
        let mut child_ids = Vec::new();
        for child in widget.children() {
            child_ids.extend(walk_widget(child.as_ref(), bounds, next_id, nodes));
        }
        return child_ids;
    }

    // Assign ID: prefer the widget's own ID, otherwise auto-generate.
    // Auto-generated IDs start at 1 and increment; widget-owned IDs are
    // typically large (atomic counter) so collisions are unlikely.
    let id = widget.widget_id().unwrap_or_else(|| {
        let id = *next_id;
        *next_id += 1;
        id
    });
    let node_id = NodeId(id);

    // Recurse into children.
    let mut child_ids = Vec::new();
    for child in widget.children() {
        child_ids.extend(walk_widget(child.as_ref(), bounds, next_id, nodes));
    }

    // Build the AccessKit node.
    // NOTE: Per-widget bounds are not yet tracked by the layout system,
    // so we currently use the parent's bounds as a placeholder. This is
    // sufficient for tree navigation; precise hit-testing will require
    // storing layout rects in a future release.
    let ak_bounds = to_ak_rect(bounds);
    let node = info.build_node(child_ids, ak_bounds);
    nodes.push((node_id, node));

    vec![node_id]
}

fn to_ak_rect(r: Rect) -> accesskit::Rect {
    accesskit::Rect::new(
        r.x1 as f64,
        r.y1 as f64,
        r.x2 as f64,
        r.y2 as f64,
    )
}

/// Convert an AccessKit action request into a WidgetEvent, if applicable.
pub(crate) fn action_to_widget_event(request: &ActionRequest) -> Option<aurora_core::kmi::WidgetEvent> {
    match request.action {
        Action::Focus => {
            let id = request.target_node.0;
            Some(aurora_core::kmi::WidgetEvent::Focus(id, false))
        }
        Action::Click => {
            // Synthesize a click at (0,0) — the widget will handle it by ID
            let id = request.target_node.0;
            Some(aurora_core::kmi::WidgetEvent::Focus(id, false))
        }
        Action::Blur => {
            let id = request.target_node.0;
            Some(aurora_core::kmi::WidgetEvent::Blur(id))
        }
        _ => None,
    }
}
