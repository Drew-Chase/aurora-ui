use std::ffi::c_void;
use std::mem::size_of;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE,
};
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCLIENT, HTLEFT, HTRIGHT, HTTOP,
    HTTOPLEFT, HTTOPRIGHT, WM_NCCALCSIZE, WM_NCHITTEST,
};

const BORDER_WIDTH: i32 = 8;
const SUBCLASS_ID: usize = 1;

/// Applies Windows 11 custom frame styling to a winit window.
///
/// This enables DWM rounded corners, a drop shadow, and installs a
/// window subclass for edge-resize hit testing.
pub fn apply(window: &winit::window::Window) {
    let Some(hwnd) = get_hwnd(window) else {
        log::warn!("Failed to extract HWND from winit window");
        return;
    };
    apply_rounded_corners(hwnd);
    apply_drop_shadow(hwnd);
    install_custom_frame(hwnd);
}

fn get_hwnd(window: &winit::window::Window) -> Option<HWND> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Some(HWND(h.hwnd.get() as *mut c_void)),
        _ => None,
    }
}

fn apply_rounded_corners(hwnd: HWND) {
    let preference = DWMWCP_ROUND;
    unsafe {
        if let Err(e) = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const DWM_WINDOW_CORNER_PREFERENCE as *const c_void,
            size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        ) {
            log::warn!("DwmSetWindowAttribute (rounded corners) failed: {e}");
        }
    }
}

fn apply_drop_shadow(hwnd: HWND) {
    let margins = MARGINS {
        cxLeftWidth: 0,
        cxRightWidth: 0,
        cyTopHeight: 0,
        cyBottomHeight: 1,
    };
    unsafe {
        if let Err(e) = DwmExtendFrameIntoClientArea(hwnd, &margins) {
            log::warn!("DwmExtendFrameIntoClientArea (drop shadow) failed: {e}");
        }
    }
}

fn install_custom_frame(hwnd: HWND) {
    unsafe {
        if !SetWindowSubclass(hwnd, Some(custom_frame_proc), SUBCLASS_ID, 0).as_bool() {
            log::warn!("SetWindowSubclass (custom frame) failed");
        }
    }
}

// SAFETY: This callback is registered via SetWindowSubclass and invoked by the
// Windows message loop with a valid hwnd. lparam encodes screen coordinates as
// (x | (y << 16)) per the WM_NCHITTEST convention.
unsafe extern "system" fn custom_frame_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid_subclass: usize,
    _ref_data: usize,
) -> LRESULT {
    match msg {
        WM_NCCALCSIZE if wparam.0 != 0 => LRESULT(0),
        WM_NCHITTEST => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut rect = RECT::default();
            if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
                return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
            }

            let left = x - rect.left < BORDER_WIDTH;
            let right = rect.right - x <= BORDER_WIDTH;
            let top = y - rect.top < BORDER_WIDTH;
            let bottom = rect.bottom - y <= BORDER_WIDTH;

            let hit = if top && left {
                HTTOPLEFT
            } else if top && right {
                HTTOPRIGHT
            } else if bottom && left {
                HTBOTTOMLEFT
            } else if bottom && right {
                HTBOTTOMRIGHT
            } else if top {
                HTTOP
            } else if bottom {
                HTBOTTOM
            } else if left {
                HTLEFT
            } else if right {
                HTRIGHT
            } else {
                HTCLIENT
            };

            LRESULT(hit as isize)
        }
        _ => unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) },
    }
}
