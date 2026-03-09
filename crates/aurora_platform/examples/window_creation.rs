use aurora_platform::app::App;

fn main() {
    App::new()
        .title("Hello, Aurora")
        .size((800, 600))
        .run(|_app, frame_info| {
            println!("App started with frame info: {:?}", frame_info);
        })
        .expect("Failed to start application");
}
