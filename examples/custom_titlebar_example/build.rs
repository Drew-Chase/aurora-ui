fn main() {
    #[cfg(target_os = "windows")]
    {
        let png_path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
        if std::path::Path::new(png_path).exists() {
            let img = image::open(png_path).expect("Failed to open icon.png");
            let ico_path =
                std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("icon.ico");
            img.save(&ico_path).expect("Failed to save icon as ICO");

            let mut res = winresource::WindowsResource::new();
            res.set_icon(ico_path.to_str().unwrap());
            res.compile().expect("Failed to compile Windows resources");
        }
    }
}
