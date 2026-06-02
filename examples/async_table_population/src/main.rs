use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use aurora_ui::aurora_widgets::components::data_table::DataTable;
use aurora_ui::aurora_widgets::components::typography::Typography;
use aurora_ui::prelude::*;
use std::process::ExitCode;

use crate::app_window::load_csv_file;

mod app_window;

enum CsvPayload {
    Data {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        source: String,
    },
    Error(String),
}

fn main() -> Result<ExitCode> {
    let (tx, rx) = mpsc::channel::<CsvPayload>();

    // Wrap a sender clone for the menu callback (requires Fn + Send + Sync).
    let menu_tx: Arc<Mutex<mpsc::Sender<CsvPayload>>> = Arc::new(Mutex::new(tx.clone()));

    // The render closure captures this sender directly (no Sync needed).
    let render_tx = tx;

    App::new()
        .size((800, 600))
        .min_size((800, 600))
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Roboto"))
        .title("Async Table")
        .menu(
            NativeMenu::new()
                .submenu(
                    SubmenuBuilder::new("File")
                        .item(
                            MenuItemBuilder::new("open", "Open CSV...")
                                .accelerator(MenuAccelerator::new(MenuModifiers::ctrl(), 'O'))
                                .build(),
                        )
                        .separator()
                        .item(NativeMenuItem::Predefined(PredefinedItem::Quit))
                        .build(),
                )
                .on_item_click(move |id| {
                    if id == "open" {
                        let path = FileDialog::new()
                            .title("Open CSV File")
                            .filter(FileFilter::new("CSV Files", &["csv"]))
                            .filter(FileFilter::new("All Files", &["*"]))
                            .open_file();

                        if let Some(path) = path {
                            let display = path.display().to_string();
                            match load_csv_file(&path) {
                                Ok((headers, rows)) => {
                                    let source =
                                        format!("Loaded {} rows from {}", rows.len(), display);
                                    if let Ok(s) = menu_tx.lock() {
                                        let _ = s.send(CsvPayload::Data {
                                            headers,
                                            rows,
                                            source,
                                        });
                                    }
                                }
                                Err(e) => {
                                    if let Ok(s) = menu_tx.lock() {
                                        let _ = s.send(CsvPayload::Error(format!(
                                            "Failed to load {}: {}",
                                            display, e
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }),
        )
        .run({
            let mut initialized = false;
            let mut headers: Vec<String> = Vec::new();
            let mut rows: Vec<Vec<String>> = Vec::new();
            let mut status = String::from("Loading...");
            let mut dirty = true;
            let mut last_height: u32 = 0;
            let scroll = ScrollState::new();

            move |window: &mut AppWindow, frame: FrameInfo| {
                // Poll for new CSV data from menu or async load.
                while let Ok(payload) = rx.try_recv() {
                    match payload {
                        CsvPayload::Data {
                            headers: h,
                            rows: r,
                            source,
                        } => {
                            headers = h;
                            rows = r;
                            status = source;
                        }
                        CsvPayload::Error(msg) => {
                            status = msg;
                        }
                    }
                    dirty = true;
                }

                // Kick off async load of data.csv on the first frame.
                if !initialized {
                    initialized = true;
                    let spawner = window.task_spawner();
                    let tx = render_tx.clone();
                    spawner.spawn(move || match load_csv_file("./examples/async_table_population/data.csv") {
                        Ok((headers, rows)) => {
                            let source = format!("Loaded {} rows from data.csv", rows.len());
                            let _ = tx.send(CsvPayload::Data {
                                headers,
                                rows,
                                source,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(CsvPayload::Error(format!(
                                "Failed to load data.csv: {}",
                                e
                            )));
                        }
                    });
                }

                if frame.height != last_height {
                    last_height = frame.height;
                    dirty = true;
                }

                if !dirty {
                    return;
                }
                dirty = false;

                // Build the DataTable from current data.
                let mut table = DataTable::new();
                for h in &headers {
                    table = table.column(h);
                }
                for r in &rows {
                    table = table.row(r.clone());
                }

                let table_height = frame.height as f32 - 40.0;
                if table_height > 0.0 {
                    table = table.viewport_height(table_height);
                }
                table = table.scroll_state(scroll.clone());

                window.root(
                    col!()
                        .child(table)
                        .child(Typography::muted(&status)),
                );
            }
        })?;

    Ok(ExitCode::SUCCESS)
}
