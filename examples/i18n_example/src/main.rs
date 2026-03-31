#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_i18n::format;
use aurora_ui::aurora_i18n::locale::Locale;
use aurora_ui::aurora_i18n::messages::MessageBundles;
use aurora_ui::prelude::*;

fn main() {
    // Detect system locale and derive text direction automatically
    let locale = Locale::system();
    println!(
        "System locale: {} ({})",
        locale.tag(),
        if locale.direction().is_rtl() {
            "RTL"
        } else {
            "LTR"
        }
    );

    // Set up fluent message bundles for English and Arabic
    let mut bundles = MessageBundles::new("en-US");
    bundles
        .add_from_str(
            "en-US",
            "app-title = Internationalization Demo\ngreeting = Welcome to Aurora UI!\nitem-count = You have { $count } items.\nformatted-price = Price: { $price }",
        )
        .expect("valid fluent source");
    bundles
        .add_from_str(
            "ar-SA",
            "app-title = عرض التدويل\ngreeting = !مرحبًا بك في Aurora UI\nitem-count = لديك { $count } عناصر.\nformatted-price = السعر: { $price }",
        )
        .expect("valid fluent source");

    // Format the greeting for the detected locale (falls back to en-US)
    let greeting = bundles.format_for_locale(locale.tag(), "greeting", &[]);
    let title = bundles.format_for_locale(locale.tag(), "app-title", &[]);

    // Locale-aware number and date formatting
    let number = format::format_number(1234567.89, 2, &locale);
    let date = format::format_date(2025, 6, 15, &locale);
    let currency = format::format_currency(9999.99, "USD", &locale);
    let integer = format::format_integer(42000, &locale);

    // Use RTL direction for demonstration (change to locale.direction() for auto-detect)
    App::new()
        .title(&title)
        .locale(locale.tag())
        .text_direction(TextDirection::Ltr)
        .min_size((400, 500))
        .size((500, 600))
        .font(include_bytes!("../../Roboto-Regular.ttf"))
        .run(move |window, _frame_info| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(20.0))
                    // Title
                    .child(Text::new(&greeting).font_size(24.0).align(Align::Start))
                    // Number formatting section
                    .child(Text::new("Locale-Aware Formatting:").font_size(18.0))
                    .child(Text::new(format!("Number: {number}")).font_size(14.0))
                    .child(Text::new(format!("Integer: {integer}")).font_size(14.0))
                    .child(Text::new(format!("Date: {date}")).font_size(14.0))
                    .child(Text::new(format!("Currency: {currency}")).font_size(14.0))
                    // Row demonstration (children flow LTR or RTL based on direction)
                    .child(Text::new("Row Layout (follows text direction):").font_size(18.0))
                    .child(
                        row!()
                            .spacing(8.0)
                            .child(
                                BoxWidget::new()
                                    .width(80)
                                    .height(40)
                                    .background_color(Color::from_rgb(66, 133, 244))
                                    .child(
                                        Text::new("First").color(Color::WHITE).align(Align::Center),
                                    ),
                            )
                            .child(
                                BoxWidget::new()
                                    .width(80)
                                    .height(40)
                                    .background_color(Color::from_rgb(234, 67, 53))
                                    .child(
                                        Text::new("Second")
                                            .color(Color::WHITE)
                                            .align(Align::Center),
                                    ),
                            )
                            .child(
                                BoxWidget::new()
                                    .width(80)
                                    .height(40)
                                    .background_color(Color::from_rgb(52, 168, 83))
                                    .child(
                                        Text::new("Third").color(Color::WHITE).align(Align::Center),
                                    ),
                            ),
                    )
                    // Locale info
                    .child(Text::new(format!("Locale: {}", locale.tag())).font_size(12.0))
                    .child(
                        Text::new(format!(
                            "Direction: {}",
                            if locale.direction().is_rtl() {
                                "RTL"
                            } else {
                                "LTR"
                            }
                        ))
                        .font_size(12.0),
                    ),
            );
        })
        .expect("Failed to run app");
}
