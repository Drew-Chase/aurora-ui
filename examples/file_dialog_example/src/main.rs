#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    App::new()
        .title("File Dialog Example")
        .size((500, 350))
        .resizable(false)
        .use_system_fonts()
        .position(WindowPosition::Center)
        .run(|window, _frame| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(24.0))
                    .child(Text::new("File Dialog Example").font_size(22.0))
                    .child(button!("Open File").width(470).on_click(|_| {
                        println!("Open File");
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
                    .child(button!("Open Multiple Files").width(470).on_click(|_| {
                        let paths = FileDialog::new()
                            .title("Select Files")
                            .filter(FileFilter::new("Images", &["png", "jpg", "gif", "webp"]))
                            .open_files();

                        println!("Selected {} file(s)", paths.len());
                        for p in &paths {
                            println!("  - {}", p.display());
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
                    .child(button!("Pick Folder").width(470).on_click(|_| {
                        let path = FileDialog::new().title("Choose Directory").pick_folder();

                        match path {
                            Some(p) => println!("Folder: {}", p.display()),
                            None => println!("Folder pick cancelled"),
                        }
                    })),
            );
        })
        .expect("Failed to run app");
}
