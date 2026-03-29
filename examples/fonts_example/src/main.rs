aurora_fonts::font_families!("Roboto");

fn main() {
    // Named weight access
    let regular = FontFamily::roboto().regular();
    println!("Roboto Regular: {} bytes", regular.len());

    let bold = FontFamily::roboto().bold();
    println!("Roboto Bold: {} bytes", bold.len());

    // Italic
    let italic = FontFamily::roboto().italic();
    println!("Roboto Italic: {} bytes", italic.len());

    // Numeric weight access
    let w300 = FontFamily::roboto().weight(300);
    println!(
        "Roboto 300: {}",
        w300.map_or("not available".into(), |b| format!("{} bytes", b.len()))
    );

    // Multiple weights
    let fonts = FontFamily::roboto().with_weights(&[100, 400, 700]);
    println!(
        "with_weights([100, 400, 700]): {} fonts loaded",
        fonts.len()
    );

    // Dynamic family lookup
    let dyn_font = FontFamily::by_name("roboto").and_then(|f| f.weight(400));
    println!(
        "Dynamic lookup: {}",
        dyn_font.map_or("not found".into(), |b| format!("{} bytes", b.len()))
    );
}
