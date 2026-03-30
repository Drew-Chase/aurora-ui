#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    let mut initialized = false;

    App::new()
        .title("Drag & Drop Example")
        .size((700, 500))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(move |window, _frame| {
            if !initialized {
                window.root(
                    col!()
                        .spacing(16.0)
                        .padding(Edges::all(24.0))
                        .child(
                            Text::new("Drag & Drop")
                                .font_size(24.0)
                                .font_weight(FontWeight::Bold),
                        )
                        .child(
                            Text::new(
                                "Drag the colored boxes below. Drop zones highlight when hovered.",
                            )
                            .font_size(14.0)
                            .color(Color::from_rgb(120, 120, 120)),
                        )
                        .child(internal_dnd_demo())
                        .child(
                            Text::new("File Drop")
                                .font_size(18.0)
                                .font_weight(FontWeight::SemiBold),
                        )
                        .child(
                            Text::new("Drag files from your OS file manager onto the zone below.")
                                .font_size(14.0)
                                .color(Color::from_rgb(120, 120, 120)),
                        )
                        .child(file_drop_demo()),
                );
                initialized = true;
            }
        })
        .expect("Failed to run app");
}

// ─── Internal drag-and-drop demo ────────────────────────────────────────────

#[derive(Clone)]
struct DndState {
    log: Vec<String>,
}

impl Default for DndState {
    fn default() -> Self {
        Self {
            log: vec!["Drag a colored box onto a drop zone.".into()],
        }
    }
}

fn internal_dnd_demo() -> impl Widget {
    Composite::new(DndState::default(), move |state, setter| {
        let s1 = setter.clone();
        let s2 = setter.clone();
        let s3 = setter.clone();

        let last_log = state.log.last().cloned().unwrap_or_default();

        Box::new(
            col!()
                .spacing(12.0)
                .child(
                    Text::new("Draggable Items")
                        .font_size(16.0)
                        .font_weight(FontWeight::SemiBold),
                )
                .child(
                    row!()
                        .spacing(12.0)
                        .child(draggable_card("Red", Color::from_rgb(220, 50, 50)))
                        .child(draggable_card("Green", Color::from_rgb(50, 180, 50)))
                        .child(draggable_card("Blue", Color::from_rgb(50, 100, 220))),
                )
                .child(
                    Text::new("Drop Zones")
                        .font_size(16.0)
                        .font_weight(FontWeight::SemiBold),
                )
                .child(
                    row!()
                        .spacing(12.0)
                        .child(drop_target("Zone A", s1))
                        .child(drop_target("Zone B", s2))
                        .child(drop_target("Zone C", s3)),
                )
                .child(
                    BoxWidget::new()
                        .background_color(Color::from_rgba(0, 0, 0, 15))
                        .corners(Corners::all(6.0))
                        .padding(Edges::all(10.0))
                        .child(
                            Text::new(last_log)
                                .font_size(13.0)
                                .color(Color::from_rgb(80, 80, 80)),
                        ),
                ),
        )
    })
}

fn draggable_card(label: &str, color: Color) -> impl Widget {
    let label_owned = label.to_string();
    Draggable::new()
        .ghost_color(Color::from_rgba(color.red, color.green, color.blue, 100))
        .ghost_corners(Corners::all(8.0))
        .on_drag_start(move |origin| {
            println!(
                "[{}] drag started at ({:.0}, {:.0})",
                label_owned, origin.x, origin.y
            );
        })
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
            println!("{}", msg);
            setter.set(move |state| state.log.push(msg.clone()));
        })
        .child(
            BoxWidget::new()
                .background_color(Color::from_rgba(0, 0, 0, 10))
                .width(180)
                .height(100)
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
                                .color(Color::from_rgb(80, 80, 80)),
                        )
                        .child(
                            Text::new("Drop here")
                                .font_size(12.0)
                                .color(Color::from_rgb(140, 140, 140)),
                        ),
                ),
        )
}

// ─── File drop demo ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct FileDropState {
    message: String,
}

impl Default for FileDropState {
    fn default() -> Self {
        Self {
            message: "No files dropped yet.".into(),
        }
    }
}

fn file_drop_demo() -> impl Widget {
    Composite::new(FileDropState::default(), move |state, setter| {
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
                    println!("{}", msg);
                    s.set(move |state| state.message = msg.clone());
                })
                .child(
                    BoxWidget::new()
                        .background_color(Color::from_rgba(0, 0, 0, 8))
                        .height(100)
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
                                        .color(Color::from_rgb(80, 80, 80))
                                        .align(Align::Center),
                                )
                                .child(
                                    Text::new(state.message.clone())
                                        .font_size(13.0)
                                        .color(Color::from_rgb(120, 120, 120))
                                        .align(Align::Center),
                                ),
                        ),
                ),
        )
    })
}
