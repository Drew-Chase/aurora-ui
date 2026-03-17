#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;

use aurora_ui::prelude::*;

fn main() {
    let username = Rc::new(RefCell::new(String::new()));
    let email = Rc::new(RefCell::new(String::new()));
    let password = Rc::new(RefCell::new(String::new()));

    // Clones for on_change closures
    let username_w = username.clone();
    let email_w = email.clone();
    let password_w = password.clone();

    // Clones for submit button
    let username_r = username.clone();
    let email_r = email.clone();
    let password_r = password.clone();

    App::new()
        .title("Text Input Example")
        .size((400, 300))
        .position(WindowPosition::Center)
        .background_color(hsl!(0, 0.0, 0.9))
        .resizable(false)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI").size(14.0))
        .run(move |window, _frame| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(20.0))
                    .align(Align::Center)
                    .child(
                        Text::new("Login")
                            .font_size(24.0)
                            .font_weight(FontWeight::Bold)
                            .height(36.0),
                    )
                    .child(
                        TextInput::new()
                            .placeholder("Username")
                            .height(34.0)
                            .corners(Corners::all(6.0))
                            .padding(Edges::symmetric(8.0, 10.0))
                            .on_change({
                                let username = username_w.clone();
                                move |value| *username.borrow_mut() = value.to_string()
                            }),
                    )
                    .child(
                        TextInput::new()
                            .placeholder("Email address")
                            .height(34.0)
                            .corners(Corners::all(6.0))
                            .padding(Edges::symmetric(8.0, 10.0))
                            .on_change({
                                let email = email_w.clone();
                                move |value| *email.borrow_mut() = value.to_string()
                            }),
                    )
                    .child(
                        TextInput::new()
                            .placeholder("Password")
                            .height(34.0)
                            .corners(Corners::all(6.0))
                            .padding(Edges::symmetric(8.0, 10.0))
                            .on_change({
                                let password = password_w.clone();
                                move |value| *password.borrow_mut() = value.to_string()
                            }),
                    )
                    .child(button(ButtonOptions {
                        text_options: Text::new("Login")
                            .align(Align::Center)
                            .justify(Justify::Center)
                            .color(Color::WHITE),
                        width: 220,
                        height: 40,
                        border_radius: Corners::all(26.0),
                        background_color: hsl!(0, 0.85, 0.64),
                        hover_background_color: hsl!(0, 0.64, 0.6),
                        text_color: Color::WHITE,
                        text_hover_color: Color::WHITE,
                        on_click: {
                            let username = username_r.clone();
                            let email = email_r.clone();
                            let password = password_r.clone();
                            Box::new(move |_| {
                                println!(
                                    "Logging in with Username: {}, Email: {}, Password: {}",
                                    username.borrow(),
                                    email.borrow(),
                                    password.borrow(),
                                );
                            })
                        },
                        ..Default::default()
                    })),
            );
        })
        .expect("Failed to run app");
}
