pub mod composite_widget;
pub mod full_widget;
use aurora_ui::prelude::*;

fn main() -> Result<(), AppError>{
    App::new()
        .title("Custom Widget Example")
        .position(WindowPosition::Center)
        .use_system_fonts()
        .run(|window, _frame_info|{
            window.root(
                col!()
            )
        })
}
