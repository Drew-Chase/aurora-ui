aurora_iconify::icon_sets!(
    // Every icon gets a method, but only the ones this crate refers to are
    // downloaded and embedded.
    "mage",
    // Icons listed here are always embedded, for names only known at runtime.
    "lucide": ["arrow-left", "arrow-right"],
    // A 15,000+ icon set costs nothing until one of its icons is used.
    // (`"material-symbols": all` would embed the whole set instead.)
    "material-symbols",
);

fn main() {
    // Type-safe access
    let svg = Icon::mage().calendar_2_fill();
    println!("calendar-2-fill SVG length: {} bytes", svg.len());
    println!("First 100 chars: {}...", &svg[..100.min(svg.len())]);

    let home = Icon::material_symbols().home();
    println!("\nmaterial-symbols home SVG length: {} bytes", home.len());

    // Dynamic access by name: found because the name appears in this crate
    let svg2 = Icon::mage().by_name("chart-fill");
    println!("\nchart-fill found: {}", svg2.is_some());

    // A name built at runtime is found because it is listed in `icon_sets!`
    let direction = if std::env::args().count() > 1 {
        "left"
    } else {
        "right"
    };
    let arrow = Icon::lucide().by_name(&format!("arrow-{direction}"));
    println!("arrow-{direction} found: {}", arrow.is_some());

    // Dynamic set lookup
    let svg3 = Icon::from_set("mage").and_then(|set| set.by_name("calendar-2-fill"));
    println!("from_set lookup found: {}", svg3.is_some());

    // Non-existent icon
    let missing = Icon::mage().by_name("does-not-exist");
    println!("missing icon: {:?}", missing);
}
