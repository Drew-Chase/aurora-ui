use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.18;

type SelectCallback = Box<dyn FnMut(&[usize])>;

/// A node in a [`TreeView`] hierarchy.
///
/// Branches can contain child nodes and be expanded/collapsed.
/// Leaves are terminal items with no children.
pub struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
    expanded: bool,
    /// 0.0 = collapsed, 1.0 = expanded
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
}

impl TreeNode {
    /// Creates a branch node with a label and children built by the closure.
    pub fn branch(label: impl Into<String>, build: impl FnOnce(&mut TreeNodeBuilder)) -> Self {
        let mut builder = TreeNodeBuilder {
            children: Vec::new(),
        };
        build(&mut builder);
        Self {
            label: label.into(),
            children: builder.children,
            expanded: false,
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
        }
    }

    /// Creates a leaf node with no children.
    pub fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
        }
    }

    fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }

    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }

    fn is_animating(&self) -> bool {
        if self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION {
            return true;
        }
        self.children.iter().any(|c| c.is_animating())
    }
}

/// Builder helper for constructing child nodes inside a branch.
pub struct TreeNodeBuilder {
    children: Vec<TreeNode>,
}

impl TreeNodeBuilder {
    /// Adds a branch child with nested children.
    pub fn branch(
        &mut self,
        label: impl Into<String>,
        build: impl FnOnce(&mut TreeNodeBuilder),
    ) -> &mut Self {
        self.children.push(TreeNode::branch(label, build));
        self
    }

    /// Adds a leaf child.
    pub fn leaf(&mut self, label: impl Into<String>) -> &mut Self {
        self.children.push(TreeNode::leaf(label));
        self
    }
}

/// A hierarchical tree display with expand/collapse, selection, and animation.
///
/// # Example
/// ```ignore
/// TreeView::new()
///     .node(TreeNode::branch("src", |b| {
///         b.branch("components", |b| {
///             b.leaf("Button.rs");
///             b.leaf("Input.rs");
///         });
///         b.leaf("main.rs");
///     }))
///     .node(TreeNode::leaf("Cargo.toml"))
///     .indent(20.0)
///     .item_height(28.0)
///     .on_select(|path| println!("selected: {path:?}"))
/// ```
pub struct TreeView {
    roots: Vec<TreeNode>,
    indent: f32,
    item_height: f32,
    selected: Option<Vec<usize>>,
    hover_path: Option<Vec<usize>>,
    show_lines: bool,
    line_color: Color,
    selected_bg: Color,
    hover_bg: Color,
    text_color: Color,
    icon_color: Color,
    width: Option<f32>,
    on_select: Option<SelectCallback>,
    text_layouts: Vec<CachedLayout>,
}

struct CachedLayout {
    layout: Option<aurora_text::text_layout::TextLayout>,
    depth: usize,
    path: Vec<usize>,
    is_branch: bool,
    expanded: bool,
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            indent: 20.0,
            item_height: 28.0,
            selected: None,
            hover_path: None,
            show_lines: false,
            line_color: colors::border(),
            selected_bg: colors::accent(),
            hover_bg: colors::accent().opacity(0.5),
            text_color: colors::foreground(),
            icon_color: colors::muted_foreground(),
            width: None,
            on_select: None,
            text_layouts: Vec::new(),
        }
    }

    /// Adds a root-level node to the tree.
    pub fn node(mut self, node: TreeNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Sets the indentation per nesting level (pixels).
    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    /// Sets the height of each tree item row.
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Sets the initially selected path (indices from root to leaf).
    pub fn selected(mut self, path: Vec<usize>) -> Self {
        self.selected = Some(path);
        self
    }

    /// Whether to draw connecting lines between nodes.
    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    /// Sets the color of connecting lines.
    pub fn line_color(mut self, color: Color) -> Self {
        self.line_color = color;
        self
    }

    /// Sets the background color for the selected item.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Sets the background color on hover.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the text color for node labels.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Sets the color for expand/collapse icons.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = color;
        self
    }

    /// Sets a fixed width for the tree view.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets the callback when a node is selected. Receives the path as indices.
    pub fn on_select(mut self, cb: impl FnMut(&[usize]) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    /// Pre-expands nodes at the given paths.
    pub fn expanded(mut self, paths: &[&[usize]]) -> Self {
        for path in paths {
            if let Some(node) = self.node_at_path_mut(path) {
                node.expanded = true;
                node.anim_from = 1.0;
                node.anim_to = 1.0;
            }
        }
        self
    }

    fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let mut current = self.roots.get_mut(path[0])?;
        for &idx in &path[1..] {
            current = current.children.get_mut(idx)?;
        }
        Some(current)
    }

    /// Flattens the visible tree into a list for rendering.
    fn flatten_visible(
        nodes: &[TreeNode],
        depth: usize,
        path_prefix: &[usize],
    ) -> Vec<FlatNode> {
        let mut result = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            let mut path = path_prefix.to_vec();
            path.push(i);
            result.push(FlatNode {
                depth,
                path: path.clone(),
                is_branch: node.is_branch(),
                expanded: node.expanded,
                label: node.label.clone(),
            });
            if node.is_branch() {
                let t = node.current_t();
                if t > 0.0 || node.expanded {
                    let children = Self::flatten_visible(&node.children, depth + 1, &path);
                    result.extend(children);
                }
            }
        }
        result
    }

    fn toggle_node(&mut self, path: &[usize]) {
        if let Some(node) = self.node_at_path_mut(path)
            && node.is_branch()
        {
            let new_state = !node.expanded;
            node.expanded = new_state;
            node.anim_from = node.current_t();
            node.anim_to = if new_state { 1.0 } else { 0.0 };
            node.anim_start = Instant::now();
        }
    }

    fn any_animating(nodes: &[TreeNode]) -> bool {
        nodes
            .iter()
            .any(|n| n.is_animating() || Self::any_animating(&n.children))
    }
}

struct FlatNode {
    depth: usize,
    path: Vec<usize>,
    is_branch: bool,
    expanded: bool,
    label: String,
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TreeView {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let visible = Self::flatten_visible(&self.roots, 0, &[]);

        self.text_layouts.clear();
        for flat in &visible {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(13.0);
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &flat.label,
                &opts,
                self.text_color,
                None,
            );
            self.text_layouts.push(CachedLayout {
                layout: Some(tl),
                depth: flat.depth,
                path: flat.path.clone(),
                is_branch: flat.is_branch,
                expanded: flat.expanded,
            });
        }

        let total_h = visible.len() as f32 * self.item_height;
        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut y = rect.y1;

        for cached in &self.text_layouts {
            let item_rect = Rect::new(rect.x1, y, rect.x2, y + self.item_height);
            let x_offset = rect.x1 + cached.depth as f32 * self.indent;

            // Selection / hover background
            let is_selected = self
                .selected
                .as_ref()
                .is_some_and(|p| *p == cached.path);
            let is_hovered = self
                .hover_path
                .as_ref()
                .is_some_and(|p| *p == cached.path);

            if is_selected {
                canvas.fill_rect(item_rect, self.selected_bg);
            } else if is_hovered {
                canvas.fill_rect(item_rect, self.hover_bg);
            }

            // Connecting lines
            if self.show_lines && cached.depth > 0 {
                let line_x = rect.x1 + (cached.depth as f32 - 0.5) * self.indent;
                let mid_y = y + self.item_height / 2.0;
                // Vertical line segment
                canvas.draw_line(
                    Point::new(line_x, y),
                    Point::new(line_x, mid_y),
                    1.0,
                    self.line_color,
                );
                // Horizontal line to node
                canvas.draw_line(
                    Point::new(line_x, mid_y),
                    Point::new(x_offset + 4.0, mid_y),
                    1.0,
                    self.line_color,
                );
            }

            // Expand/collapse chevron for branches
            if cached.is_branch {
                let icon_x = x_offset + 6.0;
                let icon_cy = y + self.item_height / 2.0;

                if cached.expanded {
                    // Down-pointing chevron
                    canvas.draw_line(
                        Point::new(icon_x - 3.0, icon_cy - 2.0),
                        Point::new(icon_x, icon_cy + 2.0),
                        1.0,
                        self.icon_color,
                    );
                    canvas.draw_line(
                        Point::new(icon_x, icon_cy + 2.0),
                        Point::new(icon_x + 3.0, icon_cy - 2.0),
                        1.0,
                        self.icon_color,
                    );
                } else {
                    // Right-pointing chevron
                    canvas.draw_line(
                        Point::new(icon_x - 2.0, icon_cy - 3.0),
                        Point::new(icon_x + 2.0, icon_cy),
                        1.0,
                        self.icon_color,
                    );
                    canvas.draw_line(
                        Point::new(icon_x + 2.0, icon_cy),
                        Point::new(icon_x - 2.0, icon_cy + 3.0),
                        1.0,
                        self.icon_color,
                    );
                }
            }

            // Label text
            if let Some(ref tl) = cached.layout {
                let text_x = x_offset + if cached.is_branch { 16.0 } else { 8.0 };
                let text_s = tl.size();
                let text_y = y + (self.item_height - text_s.height) / 2.0;
                canvas.draw_text(tl, text_x as i32, text_y as i32);
            }

            y += self.item_height;
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                let row = ((e.position.y - rect.y1) / self.item_height) as usize;
                if let Some(cached) = self.text_layouts.get(row) {
                    let path = cached.path.clone();
                    let x_offset = rect.x1 + cached.depth as f32 * self.indent;

                    if cached.is_branch && e.position.x < x_offset + 16.0 {
                        // Clicked on chevron area — toggle expand
                        self.toggle_node(&path);
                    } else {
                        // Clicked on label — select + toggle if branch
                        if cached.is_branch {
                            self.toggle_node(&path);
                        }
                        self.selected = Some(path.clone());
                        if let Some(ref mut cb) = self.on_select {
                            cb(&path);
                        }
                    }

                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                let row = ((pos.y - rect.y1) / self.item_height) as usize;
                self.hover_path = self.text_layouts.get(row).map(|c| c.path.clone());
                EventResponse {
                    status: EventStatus::Consumed,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(_)) => {
                if self.hover_path.is_some() {
                    self.hover_path = None;
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }

    fn needs_animation(&self) -> bool {
        Self::any_animating(&self.roots)
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_node_branch_has_children() {
        let node = TreeNode::branch("root", |b| {
            b.leaf("child1");
            b.leaf("child2");
        });
        assert!(node.is_branch());
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].label, "child1");
        assert_eq!(node.children[1].label, "child2");
    }

    #[test]
    fn tree_node_leaf_has_no_children() {
        let node = TreeNode::leaf("item");
        assert!(!node.is_branch());
        assert!(node.children.is_empty());
    }

    #[test]
    fn nested_branches() {
        let node = TreeNode::branch("root", |b| {
            b.branch("sub", |b| {
                b.leaf("deep");
            });
        });
        assert_eq!(node.children.len(), 1);
        assert!(node.children[0].is_branch());
        assert_eq!(node.children[0].children[0].label, "deep");
    }

    #[test]
    fn flatten_visible_collapsed() {
        let roots = vec![TreeNode::branch("root", |b| {
            b.leaf("child1");
            b.leaf("child2");
        })];
        // Root is collapsed, so only root is visible
        let flat = TreeView::flatten_visible(&roots, 0, &[]);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].label, "root");
        assert_eq!(flat[0].depth, 0);
    }

    #[test]
    fn flatten_visible_expanded() {
        let mut roots = vec![TreeNode::branch("root", |b| {
            b.leaf("child1");
            b.leaf("child2");
        })];
        roots[0].expanded = true;
        roots[0].anim_from = 1.0;
        roots[0].anim_to = 1.0;

        let flat = TreeView::flatten_visible(&roots, 0, &[]);
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].label, "root");
        assert_eq!(flat[1].label, "child1");
        assert_eq!(flat[1].depth, 1);
        assert_eq!(flat[2].label, "child2");
    }

    #[test]
    fn tree_view_default_values() {
        let tv = TreeView::new();
        assert_eq!(tv.indent, 20.0);
        assert_eq!(tv.item_height, 28.0);
        assert!(tv.selected.is_none());
        assert!(!tv.show_lines);
    }

    #[test]
    fn builder_methods_chain() {
        let tv = TreeView::new()
            .indent(24.0)
            .item_height(32.0)
            .show_lines(true)
            .selected(vec![0, 1]);

        assert_eq!(tv.indent, 24.0);
        assert_eq!(tv.item_height, 32.0);
        assert!(tv.show_lines);
        assert_eq!(tv.selected, Some(vec![0, 1]));
    }

    #[test]
    fn expanded_sets_node_state() {
        let tv = TreeView::new()
            .node(TreeNode::branch("root", |b| {
                b.branch("sub", |b| {
                    b.leaf("leaf");
                });
            }))
            .expanded(&[&[0]]);

        assert!(tv.roots[0].expanded);
    }

    #[test]
    fn empty_tree_has_no_nodes() {
        let tv = TreeView::new();
        assert!(tv.roots.is_empty());
        let flat = TreeView::flatten_visible(&tv.roots, 0, &[]);
        assert!(flat.is_empty());
    }

    #[test]
    fn multiple_roots() {
        let tv = TreeView::new()
            .node(TreeNode::leaf("file1.rs"))
            .node(TreeNode::leaf("file2.rs"))
            .node(TreeNode::leaf("file3.rs"));

        assert_eq!(tv.roots.len(), 3);
        let flat = TreeView::flatten_visible(&tv.roots, 0, &[]);
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].path, vec![0]);
        assert_eq!(flat[1].path, vec![1]);
        assert_eq!(flat[2].path, vec![2]);
    }

    #[test]
    fn toggle_expand_collapse() {
        let mut tv = TreeView::new().node(TreeNode::branch("root", |b| {
            b.leaf("child");
        }));

        assert!(!tv.roots[0].expanded);
        tv.toggle_node(&[0]);
        assert!(tv.roots[0].expanded);
        tv.toggle_node(&[0]);
        assert!(!tv.roots[0].expanded);
    }

    #[test]
    fn toggle_leaf_is_noop() {
        let mut tv = TreeView::new().node(TreeNode::leaf("item"));
        tv.toggle_node(&[0]);
        // Leaf stays not expanded (it has no children)
        assert!(!tv.roots[0].expanded);
    }

    #[test]
    fn deeply_nested_flatten() {
        let mut roots = vec![TreeNode::branch("a", |b| {
            b.branch("b", |b| {
                b.branch("c", |b| {
                    b.leaf("d");
                });
            });
        })];
        // Expand all levels
        roots[0].expanded = true;
        roots[0].anim_to = 1.0;
        roots[0].anim_from = 1.0;
        roots[0].children[0].expanded = true;
        roots[0].children[0].anim_to = 1.0;
        roots[0].children[0].anim_from = 1.0;
        roots[0].children[0].children[0].expanded = true;
        roots[0].children[0].children[0].anim_to = 1.0;
        roots[0].children[0].children[0].anim_from = 1.0;

        let flat = TreeView::flatten_visible(&roots, 0, &[]);
        assert_eq!(flat.len(), 4);
        assert_eq!(flat[0].depth, 0);
        assert_eq!(flat[1].depth, 1);
        assert_eq!(flat[2].depth, 2);
        assert_eq!(flat[3].depth, 3);
        assert_eq!(flat[3].path, vec![0, 0, 0, 0]);
    }
}
