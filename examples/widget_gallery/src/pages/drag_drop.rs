use crate::theme;
use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_drag_drop() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Drag & Drop",
            "Draggable sources, drop zones, and OS file drop support.",
        ))
        // ── Internal DnD ────────────────────────────────────────────────
        .child(crate::example_section(
            "Internal Drag & Drop",
            "Drag the colored cards onto a drop zone. A ghost preview follows the cursor.",
        ))
        .child(crate::example_card(internal_dnd_demo()))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"// Make any widget draggable
Draggable::new()
    .ghost_color(Color::from_rgba(220, 50, 50, 100))
    .on_drag_end(|origin, pos| println!("dropped at {pos:?}"))
    .child(my_card_widget)

// Accept drops
DropZone::new()
    .hover_color(Color::from_rgba(0, 120, 215, 50))
    .on_drop(|pos| println!("received at {pos:?}"))
    .child(my_target_widget)"#,
                )
                .font_size(13.0),
        )
        // ── File drop ───────────────────────────────────────────────────
        .child(crate::example_section(
            "File Drop",
            "Drag files from your OS file manager onto the zone below.",
        ))
        .child(crate::example_card(file_drop_demo()))
        .child(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"DropZone::new()
    .hover_color(Color::from_rgba(50, 180, 50, 40))
    .on_file_drop(|path, pos| {
        println!("file: {} at {pos:?}", path.display());
    })
    .child(placeholder_widget)"#,
                )
                .font_size(13.0),
        )
}

// ─── Internal DnD demo ──────────────────────────────────────────────────────

#[derive(Clone)]
struct DndState {
    log: String,
}

fn internal_dnd_demo() -> impl Widget {
    Composite::new(
        DndState {
            log: "Drag a card onto a zone.".into(),
        },
        move |state, setter| {
            let s1 = setter.clone();
            let s2 = setter.clone();
            let s3 = setter.clone();

            Box::new(
                col!()
                    .spacing(16.0)
                    .child(
                        row!()
                            .spacing(12.0)
                            .child(draggable_card("Red", Color::from_rgb(220, 50, 50)))
                            .child(draggable_card("Green", Color::from_rgb(50, 180, 50)))
                            .child(draggable_card("Blue", Color::from_rgb(50, 100, 220))),
                    )
                    .child(
                        row!()
                            .spacing(12.0)
                            .child(drop_target("Zone A", s1))
                            .child(drop_target("Zone B", s2))
                            .child(drop_target("Zone C", s3)),
                    )
                    .child(
                        Text::new(state.log.clone())
                            .font_size(13.0)
                            .color(theme::colors::muted_foreground()),
                    ),
            )
        },
    )
}

fn draggable_card(label: &str, color: Color) -> impl Widget {
    Draggable::new()
        .ghost_color(Color::from_rgba(color.red, color.green, color.blue, 100))
        .ghost_corners(Corners::all(8.0))
        .child(
            BoxWidget::new()
                .background_color(color)
                .width(80)
                .height(80)
                .corners(Corners::all(8.0))
                .padding(Edges::all(8.0))
                .child(
                    Text::new(label)
                        .font_size(14.0)
                        .font_weight(FontWeight::Bold)
                        .color(Color::WHITE),
                ),
        )
}

fn drop_target(label: &str, setter: StateSetter<DndState>) -> impl Widget {
    let label_owned = label.to_string();
    DropZone::new()
        .hover_color(Color::from_rgba(0, 120, 215, 50))
        .hover_border_color(Color::from_rgba(0, 120, 215, 220))
        .hover_border_width(2)
        .hover_corners(Corners::all(8.0))
        .on_drop(move |pos| {
            let msg = format!("Dropped on {} at ({:.0}, {:.0})", label_owned, pos.x, pos.y);
            setter.set(move |state| state.log = msg.clone());
        })
        .child(
            BoxWidget::new()
                .background_color(theme::colors::card())
                .width(160)
                .height(80)
                .corners(Corners::all(8.0))
                .padding(Edges::all(12.0))
                .child(
                    col!()
                        .align(Align::Center)
                        .justify(Justify::Center)
                        .child(
                            Text::new(label)
                                .font_size(14.0)
                                .font_weight(FontWeight::Medium)
                                .color(theme::colors::foreground()),
                        )
                        .child(
                            Text::new("Drop here")
                                .font_size(12.0)
                                .color(theme::colors::muted_foreground()),
                        ),
                ),
        )
}

// ─── File drop demo ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct FileDropState {
    message: String,
}

fn file_drop_demo() -> impl Widget {
    Composite::new(
        FileDropState {
            message: "No files dropped yet.".into(),
        },
        move |state, setter| {
            let s = setter.clone();
            Box::new(
                DropZone::new()
                    .hover_color(Color::from_rgba(50, 180, 50, 40))
                    .hover_border_color(Color::from_rgba(50, 180, 50, 200))
                    .hover_border_width(2)
                    .hover_corners(Corners::all(8.0))
                    .on_file_drop(move |path, pos| {
                        let msg = format!(
                            "Dropped: {} at ({:.0}, {:.0})",
                            path.display(),
                            pos.x,
                            pos.y
                        );
                        s.set(move |state| state.message = msg.clone());
                    })
                    .child(
                        BoxWidget::new()
                            .background_color(theme::colors::card())
                            .height(80)
                            .corners(Corners::all(8.0))
                            .padding(Edges::all(16.0))
                            .child(
                                col!()
                                    .align(Align::Center)
                                    .justify(Justify::Center)
                                    .child(
                                        Text::new("Drop files here")
                                            .font_size(16.0)
                                            .font_weight(FontWeight::Medium)
                                            .color(theme::colors::foreground())
                                            .align(Align::Center),
                                    )
                                    .child(
                                        Text::new(state.message.clone())
                                            .font_size(13.0)
                                            .color(theme::colors::muted_foreground())
                                            .align(Align::Center),
                                    ),
                            ),
                    ),
            )
        },
    )
}
