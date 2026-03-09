use aurora_core::color::Color;
use aurora_core::color::IntoColor;
use aurora_core::lerp;

fn main() {
	let color1 = "ff00ff".color(false);
	let color2 = "cc00ff".color(false);
	let color3 = "08ff00".color(false);
	let color4 = "000000".color(false);

	let mut time = 0f32;
	loop {
		let color = lerp!(time, color1, color2, color3, color4);
		println!("{}", color);
		time += 0.01;
		if time >= 1.0 {
			break;
		}
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
}
