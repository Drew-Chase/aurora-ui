fn main() {
    println!("Hello, world!");

    use aurora_ui::color::StringColor;

    let color = "ff00ff".color();       // &str
    let color = "00ff00".to_string().color();  // String
}
