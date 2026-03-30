#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("File Dialog Example")
        .size((500, 550))
        .resizable(false)
        .use_system_fonts()
        .position(WindowPosition::Center)
        .run(|window, _frame| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(24.0))
                    .child(Text::new("File Dialog Example").font_size(22.0))
                    .child(Text::new("Synchronous (blocking)").font_size(16.0))
                    .child(button!("Open File").width(470).on_click(|_| {
                        let path = FileDialog::new()
                            .title("Open File")
                            .filter(FileFilter::new("Text Files", &["txt", "md", "toml"]))
                            .filter(FileFilter::new("Rust Files", &["rs"]))
                            .filter(FileFilter::new("All Files", &["*"]))
                            .open_file();

                        match path {
                            Some(p) => println!("Opened: {}", p.display()),
                            None => println!("Open cancelled"),
                        }
                    }))
                    .child(button!("Save File").width(470).on_click(|_| {
                        let path = FileDialog::new()
                            .title("Save As")
                            .file_name("untitled.txt")
                            .filter(FileFilter::new("Text Files", &["txt"]))
                            .save_file();

                        match path {
                            Some(p) => println!("Save to: {}", p.display()),
                            None => println!("Save cancelled"),
                        }
                    }))
                    .child(Text::new("Asynchronous (non-blocking)").font_size(16.0))
                    .child(button!("Async Open File").width(470).on_click(|_| {
                        println!("Spawning async dialog — UI stays responsive!");
                        let pending = FileDialog::new()
                            .title("Async Open File")
                            .filter(FileFilter::new("All Files", &["*"]))
                            .open_file_async();

                        std::thread::spawn(move || match pending.recv().flatten() {
                            Some(p) => println!("Async opened: {}", p.display()),
                            None => println!("Async open cancelled"),
                        });
                    }))
                    .child(button!("Async Save File").width(470).on_click(|_| {
                        println!("Spawning async save dialog — UI stays responsive!");
                        let pending = FileDialog::new()
                            .title("Async Save As")
                            .file_name("untitled.txt")
                            .filter(FileFilter::new("Text Files", &["txt"]))
                            .save_file_async();

                        std::thread::spawn(move || match pending.recv().flatten() {
                            Some(p) => println!("Async save to: {}", p.display()),
                            None => println!("Async save cancelled"),
                        });
                    }))
                    .child(button!("Async Pick Folder").width(470).on_click(|_| {
                        println!("Spawning async folder picker — UI stays responsive!");
                        let pending = FileDialog::new()
                            .title("Async Choose Directory")
                            .pick_folder_async();

                        std::thread::spawn(move || match pending.recv().flatten() {
                            Some(p) => println!("Async folder: {}", p.display()),
                            None => println!("Async folder pick cancelled"),
                        });
                    })),
            );
        })
        .expect("Failed to run app");
}
