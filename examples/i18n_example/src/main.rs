#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::aurora_i18n::format;
use aurora_ui::aurora_i18n::locale::Locale;
use aurora_ui::prelude::*;

// Compile-time locale strings from TOML files — zero runtime cost
aurora_ui::aurora_lang::lang!("locales");

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

    // Select compile-time translations for the detected locale (fallback to default)
    let l = lang::by_tag(locale.tag()).unwrap_or(&lang::LANG);

    // Locale-aware number and date formatting (runtime)
    let number = format::format_number(1234567.89, 2, &locale);
    let date = format::format_date(2025, 6, 15, &locale);
    let currency = format::format_currency(9999.99, "USD", &locale);
    let integer = format::format_integer(42000, &locale);

    // Print available compiled locales
    println!("Available locales: {:?}", lang::LOCALE_TAGS);

    App::new()
        .title(l.title.window)
        .locale(locale.tag())
        .text_direction(TextDirection::Ltr)
        .min_size((400, 500))
        .size((500, 600))
        .use_system_fonts()
        .run(move |window, _frame_info| {
            window.root(
                col!()
                    .spacing(12.0)
                    .padding(Edges::all(20.0))
                    // Title from compile-time locale strings
                    .child(Text::new(l.greeting).font_size(24.0).align(Align::Start))
                    // Number formatting section (labels from compile-time, values from runtime)
                    .child(Text::new("Locale-Aware Formatting:").font_size(18.0))
                    .child(
                        Text::new(format!("{}: {number}", l.formatting.number_label))
                            .font_size(14.0),
                    )
                    .child(
                        Text::new(format!("{}: {integer}", l.formatting.integer_label))
                            .font_size(14.0),
                    )
                    .child(
                        Text::new(format!("{}: {date}", l.formatting.date_label)).font_size(14.0),
                    )
                    .child(
                        Text::new(format!("{}: {currency}", l.formatting.currency_label))
                            .font_size(14.0),
                    )
                    // Row demonstration
                    .child(Text::new(l.row_demo).font_size(18.0))
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
                    .child(
                        Text::new(format!("{}: {}", l.locale_info.locale_label, locale.tag()))
                            .font_size(12.0),
                    )
                    .child(
                        Text::new(format!(
                            "{}: {}",
                            l.locale_info.direction_label,
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
