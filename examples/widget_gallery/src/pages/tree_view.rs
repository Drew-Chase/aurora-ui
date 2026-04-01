use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_tree_view() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Tree View",
            "A hierarchical data display with expand/collapse, selection, and animated transitions.",
        ))
        .child(crate::example_section(
            "File Explorer",
            "Click a folder to expand/collapse. Click any item to select it.",
        ))
        .child(crate::example_card(
            tree_view::TreeView::new()
                .node(tree_view::TreeNode::branch("src", |b| {
                    b.branch("components", |b| {
                        b.leaf("Button.rs");
                        b.leaf("Input.rs");
                        b.leaf("TreeView.rs");
                    });
                    b.branch("layout", |b| {
                        b.leaf("Column.rs");
                        b.leaf("Row.rs");
                        b.leaf("Stack.rs");
                    });
                    b.leaf("main.rs");
                    b.leaf("lib.rs");
                }))
                .node(tree_view::TreeNode::branch("examples", |b| {
                    b.leaf("counter.rs");
                    b.leaf("gallery.rs");
                }))
                .node(tree_view::TreeNode::leaf("Cargo.toml"))
                .node(tree_view::TreeNode::leaf("README.md"))
                .expanded(&[&[0]])
                .show_lines(true)
                .width(400.0),
        ))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"TreeView::new()
    .node(TreeNode::branch("src", |b| {
        b.branch("components", |b| {
            b.leaf("Button.rs");
            b.leaf("Input.rs");
        });
        b.leaf("main.rs");
    }))
    .node(TreeNode::leaf("Cargo.toml"))
    .expanded(&[&[0]])
    .show_lines(true)
    .item_height(28.0)
    .on_select(|path| println!("{path:?}"))"#,
                )
                .font_size(13.0),
        )
        .child(crate::example_section(
            "Compact (no lines)",
            "A simpler tree without connecting lines.",
        ))
        .child(crate::example_card(
            tree_view::TreeView::new()
                .node(tree_view::TreeNode::branch("Documents", |b| {
                    b.leaf("Resume.pdf");
                    b.leaf("Cover Letter.pdf");
                }))
                .node(tree_view::TreeNode::branch("Photos", |b| {
                    b.leaf("vacation.jpg");
                    b.leaf("family.png");
                }))
                .node(tree_view::TreeNode::leaf("notes.txt"))
                .width(300.0),
        ))
}
