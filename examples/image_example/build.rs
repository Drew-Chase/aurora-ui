use std::path::Path;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let png_path = &Path::new("../../../logo.png").canonicalize().unwrap();
        if png_path.exists() {
            let img = image::open(png_path).expect("Failed to open logo.png");
            let ico_path =
                std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("icon.ico");
            img.save(&ico_path).expect("Failed to save icon as ICO");

            let mut res = winresource::WindowsResource::new();
            res.set_icon(ico_path.to_str().unwrap());
            res.compile().expect("Failed to compile Windows resources");
        } else {
            panic!("Failed to find icon: {}", png_path.display());
        }
    }
}
