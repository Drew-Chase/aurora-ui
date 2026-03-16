use windows::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{DWMTWR_WINDOW_MARGINS, DwmExtendFrameIntoClientArea},
};

pub fn apply(window: &winit::window::Window) {
//    let hwnd = get_hwnd(window);
}

fn apply_drop_shadow(hwnd: HWND) {
    let margins = DWMTWR_WINDOW_MARGINS {
        cxLeftWidth: 0,
        cxRightWidth: 0,
        cyTopHeight: 0,
        cyBottomHeight: 0,
    };
    unsafe {
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
    }
}
