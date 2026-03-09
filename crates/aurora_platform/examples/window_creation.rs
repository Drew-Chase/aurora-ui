use aurora_platform::app::App;

fn main() {
    App::new()
        .title("Hello, Aurora")
        .size((800, 600))
        .run(|_app, frame_info| {
            println!("App re-rendered with frame info: {:?}", frame_info);
        })
        .expect("Failed to start application");
}
