aurora_iconify::icon_sets!("mage");

fn main() {
    // Type-safe access
    let svg = Icon::mage().calendar_2_fill();
    println!("calendar-2-fill SVG length: {} bytes", svg.len());
    println!("First 100 chars: {}...", &svg[..100.min(svg.len())]);

    // Dynamic access by name
    let svg2 = Icon::mage().by_name("chart-fill");
    println!("\nchart-fill found: {}", svg2.is_some());

    // Dynamic set lookup
    let svg3 = Icon::from_set("mage")
        .and_then(|set| set.by_name("calendar-2-fill"));
    println!("from_set lookup found: {}", svg3.is_some());

    // Non-existent icon
    let missing = Icon::mage().by_name("does-not-exist");
    println!("missing icon: {:?}", missing);
}
