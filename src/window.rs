use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, EndPaint, GetDC, InvalidateRect, ReleaseDC, PAINTSTRUCT},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::{
            Input::KeyboardAndMouse::{
                TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT, VIRTUAL_KEY, VK_DELETE,
            },
            WindowsAndMessaging::{
                DefWindowProcW, GetClientRect, GetWindowLongPtrW, KillTimer, PostQuitMessage,
                SetTimer, SetWindowLongPtrW, GWLP_USERDATA,
                WM_CHAR, WM_CREATE, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN,
                WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_SIZE, WM_TIMER,
            },
        },
    },
};

use crate::{
    calculator::{Calculator, Op},
    layout::{compute_button_rects, hit_test, Button, ButtonId},
    renderer::{draw_frame, GdiResources},
};

const KBD_TIMER_ID: usize = 1;
const KBD_PRESS_MS: u32 = 150;

/// Map a WM_CHAR character to the ButtonId it corresponds to.
fn char_to_button_id(ch: char) -> Option<ButtonId> {
    match ch {
        '0'..='9' => Some(ButtonId::Digit(ch as u8 - b'0')),
        '.' | ',' => Some(ButtonId::Decimal),
        '+' => Some(ButtonId::Add),
        '-' => Some(ButtonId::Sub),
        '*' => Some(ButtonId::Mul),
        '/' => Some(ButtonId::Div),
        '=' | '\r' => Some(ButtonId::Equals),
        '\x08' => Some(ButtonId::Backspace),
        '\x1B' => Some(ButtonId::Clear),
        _ => None,
    }
}

fn find_button(buttons: &[Button], id: ButtonId) -> Option<usize> {
    buttons.iter().position(|b| b.id == id)
}

const WM_MOUSELEAVE: u32 = 0x02A3;


#[link(name = "user32")]
extern "system" {
    fn SetCapture(hwnd: HWND) -> HWND;
    fn ReleaseCapture() -> BOOL;
}

/// Attempt to enable dark title bar via DwmSetWindowAttribute (attribute 20, fallback 19).
/// Uses dynamic loading — safe if dwmapi.dll is absent (WinPE without DWM).
unsafe fn try_set_dark_titlebar(hwnd: HWND) {
    let dll: Vec<u16> = "dwmapi.dll\0".encode_utf16().collect();
    let Ok(hmod) = LoadLibraryW(PCWSTR(dll.as_ptr())) else { return };

    let func_name = b"DwmSetWindowAttribute\0";
    if let Some(func) = GetProcAddress(hmod, PCSTR(func_name.as_ptr())) {
        type DwmFn = unsafe extern "system" fn(HWND, u32, *const core::ffi::c_void, u32) -> i32;
        let dwm: DwmFn = core::mem::transmute(func);
        let dark: u32 = 1;
        let ptr = &dark as *const u32 as *const _;
        let sz = core::mem::size_of::<u32>() as u32;
        // Try attribute 20 (Win10 20H1+ / Win11), then 19 (older Win10 builds)
        if dwm(hwnd, 20, ptr, sz) != 0 {
            dwm(hwnd, 19, ptr, sz);
        }
    }

    // Leave dwmapi.dll loaded — it's a system DLL and won't be unloaded anyway
}

pub struct AppState {
    pub calc: Calculator,
    pub buttons: Vec<Button>,
    pub hovered: Option<usize>,
    pub pressed: Option<usize>,
    pub tracking_mouse: bool,
}

impl AppState {
    fn new(hwnd: HWND) -> Self {
        let mut rect = RECT::default();
        unsafe { let _ = GetClientRect(hwnd, &mut rect); }
        let buttons = compute_button_rects(rect.right, rect.bottom);
        Self {
            calc: Calculator::new(),
            buttons,
            hovered: None,
            pressed: None,
            tracking_mouse: false,
        }
    }
}

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hdc = GetDC(hwnd);
            let gdi = GdiResources::create(hdc);
            ReleaseDC(hwnd, hdc);

            let state = Box::new((AppState::new(hwnd), gdi));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

            try_set_dark_titlebar(hwnd);

            LRESULT(0)
        }

        WM_DESTROY => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if !ptr.is_null() {
                drop(Box::from_raw(ptr));
            }
            PostQuitMessage(0);
            LRESULT(0)
        }

        WM_SIZE => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if !ptr.is_null() {
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                (*ptr).0.buttons = compute_button_rects(rect.right, rect.bottom);
            }
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_ERASEBKGND => LRESULT(1),

        WM_PAINT => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            let state = &(*ptr).0;
            let gdi = &(*ptr).1;

            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            draw_frame(
                hdc,
                rect.right,
                rect.bottom,
                &state.buttons,
                state.hovered,
                state.pressed,
                &state.calc.display,
                &state.calc.expression,
                gdi,
            );

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            let state = &mut (*ptr).0;

            // Cancel any keyboard press timer so it doesn't clear a mouse press
            let _ = KillTimer(hwnd, KBD_TIMER_ID);

            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if let Some(idx) = hit_test(&state.buttons, x, y) {
                state.pressed = Some(idx);
                let id = state.buttons[idx].id;
                dispatch_button(&mut state.calc, id);
                SetCapture(hwnd);
            }
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_LBUTTONUP => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if !ptr.is_null() {
                let _ = KillTimer(hwnd, KBD_TIMER_ID);
                (*ptr).0.pressed = None;
            }
            let _ = ReleaseCapture();
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            let state = &mut (*ptr).0;

            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let new_hovered = hit_test(&state.buttons, x, y);

            if new_hovered != state.hovered {
                state.hovered = new_hovered;
                let _ = InvalidateRect(hwnd, None, false);
            }

            if !state.tracking_mouse {
                state.tracking_mouse = true;
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: core::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut tme);
            }
            LRESULT(0)
        }

        WM_MOUSELEAVE => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if !ptr.is_null() {
                (*ptr).0.hovered = None;
                (*ptr).0.tracking_mouse = false;
            }
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        // WM_KEYDOWN handles only VK_DELETE (no WM_CHAR equivalent).
        // All other input is handled in WM_CHAR to avoid double-firing — TranslateMessage
        // converts every other key we care about into a WM_CHAR with the correct character
        // (including Shift+8 → '*' instead of '8').
        WM_KEYDOWN => {
            let vk = VIRTUAL_KEY(wparam.0 as u16);
            if vk == VK_DELETE {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
                if !ptr.is_null() {
                    let state = &mut (*ptr).0;
                    state.calc.clear();
                    state.pressed = find_button(&state.buttons, ButtonId::Clear);
                    SetTimer(hwnd, KBD_TIMER_ID, KBD_PRESS_MS, None);
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        WM_TIMER => {
            if wparam.0 == KBD_TIMER_ID {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
                if !ptr.is_null() {
                    (*ptr).0.pressed = None;
                }
                let _ = KillTimer(hwnd, KBD_TIMER_ID);
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        WM_CHAR => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (AppState, GdiResources);
            if ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            let ch = char::from_u32(wparam.0 as u32).unwrap_or('\0');

            // Handle calculator logic
            {
                let calc = &mut (*ptr).0.calc;
                match ch {
                    '0'..='9' => calc.digit(ch as u8 - b'0'),
                    '.' | ',' => calc.decimal(),
                    '+' => calc.operator(Op::Add),
                    '-' => calc.operator(Op::Sub),
                    '*' => calc.operator(Op::Mul),
                    '/' => calc.operator(Op::Div),
                    '=' | '\r' => calc.equals(),
                    '\x08' => calc.backspace(),
                    '\x1B' => calc.clear(),
                    _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }

            // Highlight the corresponding button for KBD_PRESS_MS milliseconds
            let state = &mut (*ptr).0;
            state.pressed = char_to_button_id(ch).and_then(|id| find_button(&state.buttons, id));
            SetTimer(hwnd, KBD_TIMER_ID, KBD_PRESS_MS, None);

            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn dispatch_button(calc: &mut Calculator, id: crate::layout::ButtonId) {
    use crate::layout::ButtonId;
    match id {
        ButtonId::Digit(d) => calc.digit(d),
        ButtonId::Decimal => calc.decimal(),
        ButtonId::Add => calc.operator(Op::Add),
        ButtonId::Sub => calc.operator(Op::Sub),
        ButtonId::Mul => calc.operator(Op::Mul),
        ButtonId::Div => calc.operator(Op::Div),
        ButtonId::Equals => calc.equals(),
        ButtonId::Clear => calc.clear(),
        ButtonId::ClearEntry => calc.clear_entry(),
        ButtonId::Backspace => calc.backspace(),
    }
}
