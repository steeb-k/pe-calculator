use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{COLORREF, RECT},
        Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
            DeleteDC, DeleteObject, DrawTextW, FillRect, GetDeviceCaps, GetStockObject,
            RoundRect, SelectObject, SetBkMode, SetTextColor, HBRUSH, HDC, HFONT,
            LOGPIXELSY, NULL_PEN, SRCCOPY, TRANSPARENT, DT_CENTER, DT_RIGHT, DT_SINGLELINE,
            DT_VCENTER, DT_END_ELLIPSIS, DRAW_TEXT_FORMAT,
        },
    },
};

use crate::layout::{Button, ButtonId, DISPLAY_HEIGHT};

// GDI COLORREFs are 0x00BBGGRR
const COLOR_BG: COLORREF    = COLORREF(0x001C1C1C);
const COLOR_BTN: COLORREF   = COLORREF(0x002D2D2D);
const COLOR_BTN_HOVER: COLORREF  = COLORREF(0x003A3A3A);
const COLOR_BTN_PRESS: COLORREF  = COLORREF(0x00484848);
// #C084FC → R=0xC0, G=0x84, B=0xFC → GDI 0x00FC84C0
const COLOR_EQUALS: COLORREF       = COLORREF(0x00FC84C0);
const COLOR_EQUALS_HOVER: COLORREF = COLORREF(0x00F755A8);
const COLOR_EQUALS_PRESS: COLORREF = COLORREF(0x00E040C0);
const COLOR_TEXT: COLORREF  = COLORREF(0x00FFFFFF);
const COLOR_EXPR: COLORREF  = COLORREF(0x00888888);

// Raw constants not exported from the windows crate at this version
const FW_NORMAL: i32 = 400;
const DEFAULT_QUALITY: u32 = 0;

fn mul_div(a: i32, b: i32, c: i32) -> i32 {
    ((a as i64 * b as i64) / c as i64) as i32
}

pub struct GdiResources {
    pub bg_brush: HBRUSH,
    pub btn_brush: HBRUSH,
    pub btn_hover_brush: HBRUSH,
    pub btn_press_brush: HBRUSH,
    pub equals_brush: HBRUSH,
    pub equals_hover_brush: HBRUSH,
    pub equals_press_brush: HBRUSH,
    pub display_font: HFONT,
    pub expr_font: HFONT,
    pub button_font: HFONT,
}

impl GdiResources {
    pub fn create(hdc: HDC) -> Self {
        unsafe {
            let dpi = GetDeviceCaps(hdc, LOGPIXELSY);
            Self {
                bg_brush: CreateSolidBrush(COLOR_BG),
                btn_brush: CreateSolidBrush(COLOR_BTN),
                btn_hover_brush: CreateSolidBrush(COLOR_BTN_HOVER),
                btn_press_brush: CreateSolidBrush(COLOR_BTN_PRESS),
                equals_brush: CreateSolidBrush(COLOR_EQUALS),
                equals_hover_brush: CreateSolidBrush(COLOR_EQUALS_HOVER),
                equals_press_brush: CreateSolidBrush(COLOR_EQUALS_PRESS),
                display_font: create_font(hdc, dpi, 36),
                expr_font: create_font(hdc, dpi, 14),
                button_font: create_font(hdc, dpi, 18),
            }
        }
    }
}

impl Drop for GdiResources {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.bg_brush);
            let _ = DeleteObject(self.btn_brush);
            let _ = DeleteObject(self.btn_hover_brush);
            let _ = DeleteObject(self.btn_press_brush);
            let _ = DeleteObject(self.equals_brush);
            let _ = DeleteObject(self.equals_hover_brush);
            let _ = DeleteObject(self.equals_press_brush);
            let _ = DeleteObject(self.display_font);
            let _ = DeleteObject(self.expr_font);
            let _ = DeleteObject(self.button_font);
        }
    }
}

unsafe fn create_font(_hdc: HDC, dpi: i32, pt: i32) -> HFONT {
    let height = -mul_div(pt, dpi, 72);
    let name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    CreateFontW(
        height,
        0, 0, 0,
        FW_NORMAL,
        false.into(),
        false.into(),
        false.into(),
        0,
        0,
        0,
        DEFAULT_QUALITY,
        0,
        PCWSTR(name.as_ptr()),
    )
}

pub fn draw_frame(
    hdc: HDC,
    width: i32,
    height: i32,
    buttons: &[Button],
    hovered: Option<usize>,
    pressed: Option<usize>,
    display: &str,
    expression: &str,
    gdi: &GdiResources,
) {
    unsafe {
        let mem_dc = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, width, height);
        let old_bmp = SelectObject(mem_dc, bmp);

        let full = RECT { left: 0, top: 0, right: width, bottom: height };
        FillRect(mem_dc, &full, gdi.bg_brush);

        draw_display(mem_dc, width, display, expression, gdi);

        for (i, btn) in buttons.iter().enumerate() {
            let is_hovered = hovered == Some(i);
            let is_pressed = pressed == Some(i);
            draw_button(mem_dc, btn, is_hovered, is_pressed, gdi);
        }

        let _ = BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        SelectObject(mem_dc, old_bmp);
        let _ = DeleteObject(bmp);
        let _ = DeleteDC(mem_dc);
    }
}

unsafe fn draw_text_wide(hdc: HDC, s: &str, rect: &RECT, format: DRAW_TEXT_FORMAT) {
    let mut wide: Vec<u16> = s.encode_utf16().collect();
    let mut r = *rect;
    DrawTextW(hdc, &mut wide, &mut r, format);
}

unsafe fn draw_display(hdc: HDC, width: i32, display: &str, expression: &str, gdi: &GdiResources) {
    SetBkMode(hdc, TRANSPARENT);

    if !expression.is_empty() {
        let expr_rect = RECT {
            left: 12,
            top: 8,
            right: width - 12,
            bottom: DISPLAY_HEIGHT / 2,
        };
        SelectObject(hdc, gdi.expr_font);
        SetTextColor(hdc, COLOR_EXPR);
        draw_text_wide(hdc, expression, &expr_rect, DT_RIGHT | DT_SINGLELINE | DT_VCENTER);
    }

    let disp_rect = RECT {
        left: 12,
        top: DISPLAY_HEIGHT / 2,
        right: width - 12,
        bottom: DISPLAY_HEIGHT,
    };
    SelectObject(hdc, gdi.display_font);
    SetTextColor(hdc, COLOR_TEXT);
    draw_text_wide(hdc, display, &disp_rect, DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS);
}

unsafe fn draw_button(hdc: HDC, btn: &Button, hovered: bool, pressed: bool, gdi: &GdiResources) {
    let is_equals = btn.id == ButtonId::Equals;

    let brush = if is_equals {
        if pressed { gdi.equals_press_brush }
        else if hovered { gdi.equals_hover_brush }
        else { gdi.equals_brush }
    } else if pressed {
        gdi.btn_press_brush
    } else if hovered {
        gdi.btn_hover_brush
    } else {
        gdi.btn_brush
    };

    let old_pen = SelectObject(hdc, GetStockObject(NULL_PEN));
    let old_brush = SelectObject(hdc, brush);

    let _ = RoundRect(
        hdc,
        btn.rect.left,
        btn.rect.top,
        btn.rect.right + 1,
        btn.rect.bottom + 1,
        12,
        12,
    );

    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);

    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, COLOR_TEXT);
    SelectObject(hdc, gdi.button_font);
    draw_text_wide(hdc, btn.label, &btn.rect, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
}
