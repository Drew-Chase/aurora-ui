use std::path::PathBuf;

fn main(){
	println!("cargo:rerun-if-changed=themes.json");
	#[cfg(target_os = "windows")]
	{
		let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let png_path = manifest_dir.join("../../logo.png");
		if png_path.exists() {
			let img = image::open(&png_path).expect("Failed to open logo.png");
			let img = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
			let ico_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("icon.ico");
			img.save(&ico_path).expect("Failed to save icon as ICO");

			let mut res = winresource::WindowsResource::new();
			res.set_icon(ico_path.to_str().unwrap());
			res.compile().expect("Failed to compile Windows resources");
		} else {
			panic!("Failed to find icon: {}", png_path.display());
		}
	}
}