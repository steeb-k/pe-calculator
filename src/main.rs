#![windows_subsystem = "windows"]

mod calculator;
mod layout;
mod renderer;
mod window;

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DispatchMessageW, GetMessageW, LoadCursorW, LoadIconW, MSG,
            RegisterClassExW, ShowWindow, TranslateMessage, WNDCLASSEXW,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, SW_SHOW,
            WS_OVERLAPPED, WS_CAPTION, WS_SYSMENU, WS_MINIMIZEBOX,
            HMENU, WINDOW_EX_STYLE,
        },
        Graphics::Gdi::HBRUSH,
    },
};

#[link(name = "user32")]
extern "system" {
    fn UpdateWindow(hwnd: HWND) -> windows::Win32::Foundation::BOOL;
}

fn main() {
    unsafe {
        let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap();

        let class_name: Vec<u16> = "PECalculator\0".encode_utf16().collect();
        let window_title: Vec<u16> = "Calculator\0".encode_utf16().collect();

        // Load the icon embedded by winres at resource ID 1
        let icon = LoadIconW(hinstance, PCWSTR(1 as *const u16)).unwrap_or_default();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: icon,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: icon,
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            340,
            540,
            HWND::default(),
            HMENU::default(),
            hinstance,
            None,
        )
        .unwrap();

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
