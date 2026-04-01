#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;

/// Generates a simple 32x32 RGBA icon: a blue circle on a transparent background.
fn generate_icon() -> (Vec<u8>, u32, u32) {
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    let center = SIZE as f32 / 2.0;
    let radius = center - 2.0;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                let idx = ((y * SIZE + x) * 4) as usize;
                rgba[idx] = 60; // R
                rgba[idx + 1] = 120; // G
                rgba[idx + 2] = 220; // B
                rgba[idx + 3] = 255; // A
            }
        }
    }
    (rgba, SIZE, SIZE)
}

fn main() {
    let (icon_rgba, icon_w, icon_h) = generate_icon();

    App::new()
        .title("Tray & Menu Example")
        .size((600, 400))
        .use_system_fonts()
        .position(WindowPosition::Center)
        // Native menu bar
        .menu(
            NativeMenu::new()
                .submenu(
                    SubmenuBuilder::new("File")
                        .item(
                            MenuItemBuilder::new("new", "New")
                                .accelerator(MenuAccelerator::new(MenuModifiers::ctrl(), 'N'))
                                .build(),
                        )
                        .item(
                            MenuItemBuilder::new("open", "Open")
                                .accelerator(MenuAccelerator::new(MenuModifiers::ctrl(), 'O'))
                                .build(),
                        )
                        .separator()
                        .item(
                            MenuItemBuilder::new("save", "Save")
                                .accelerator(MenuAccelerator::new(MenuModifiers::ctrl(), 'S'))
                                .build(),
                        )
                        .separator()
                        .item(NativeMenuItem::Predefined(PredefinedItem::Quit))
                        .build(),
                )
                .submenu(
                    SubmenuBuilder::new("Edit")
                        .item(NativeMenuItem::Predefined(PredefinedItem::Undo))
                        .item(NativeMenuItem::Predefined(PredefinedItem::Redo))
                        .item(NativeMenuItem::Separator)
                        .item(NativeMenuItem::Predefined(PredefinedItem::Cut))
                        .item(NativeMenuItem::Predefined(PredefinedItem::Copy))
                        .item(NativeMenuItem::Predefined(PredefinedItem::Paste))
                        .item(NativeMenuItem::Separator)
                        .item(NativeMenuItem::Predefined(PredefinedItem::SelectAll))
                        .build(),
                )
                .submenu(
                    SubmenuBuilder::new("View")
                        .item(
                            CheckMenuItemBuilder::new("dark_mode", "Dark Mode")
                                .checked(false)
                                .build(),
                        )
                        .item(
                            CheckMenuItemBuilder::new("sidebar", "Show Sidebar")
                                .checked(true)
                                .build(),
                        )
                        .build(),
                )
                .on_item_click(|id| {
                    println!("[menu] clicked: {id}");
                }),
        )
        // System tray icon
        .tray(
            TrayConfig::new()
                .icon_rgba(icon_rgba, icon_w, icon_h)
                .tooltip("Aurora Tray Example")
                .menu(
                    NativeMenu::new()
                        .item(MenuItemBuilder::new("show", "Show Window").build())
                        .separator()
                        .item(MenuItemBuilder::new("tray_quit", "Quit").build())
                        .on_item_click(|id| {
                            println!("[tray menu] clicked: {id}");
                            if id == "tray_quit" {
                                std::process::exit(0);
                            }
                        }),
                )
                .on_click(|interaction| {
                    println!("[tray] interaction: {interaction:?}");
                }),
        )
        .run(|window, _frame| {
            window.root(
                col!()
                    .spacing(16.0)
                    .padding(Edges::all(24.0))
                    .child(
                        Text::new("Tray & Native Menu Demo")
                            .font_size(24.0)
                            .align(Align::Center),
                    )
                    .child(
                        Text::new("This window has a native OS menu bar (File, Edit, View).")
                            .font_size(14.0),
                    )
                    .child(
                        Text::new("A system tray icon is visible in the notification area.")
                            .font_size(14.0),
                    )
                    .child(
                        Text::new("Right-click the tray icon for its context menu.")
                            .font_size(14.0),
                    )
                    .child(
                        Text::new("Menu and tray events are printed to the console.")
                            .font_size(14.0),
                    ),
            );
        })
        .expect("Failed to run app");
}
