use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonId {
    Digit(u8),
    Decimal,
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    Clear,
    ClearEntry,
    Backspace,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonDef {
    pub id: ButtonId,
    pub label: &'static str,
    pub col_span: u8,
    pub col: u8,
    pub row: u8,
}

#[derive(Debug, Clone)]
pub struct Button {
    pub id: ButtonId,
    pub label: &'static str,
    pub rect: RECT,
}

pub const BUTTON_GRID: &[ButtonDef] = &[
    ButtonDef { id: ButtonId::Clear,      label: "C",  col_span: 1, col: 0, row: 0 },
    ButtonDef { id: ButtonId::ClearEntry, label: "CE", col_span: 1, col: 1, row: 0 },
    ButtonDef { id: ButtonId::Backspace,  label: "\u{232B}", col_span: 1, col: 2, row: 0 },
    ButtonDef { id: ButtonId::Div,        label: "\u{00F7}", col_span: 1, col: 3, row: 0 },

    ButtonDef { id: ButtonId::Digit(7), label: "7", col_span: 1, col: 0, row: 1 },
    ButtonDef { id: ButtonId::Digit(8), label: "8", col_span: 1, col: 1, row: 1 },
    ButtonDef { id: ButtonId::Digit(9), label: "9", col_span: 1, col: 2, row: 1 },
    ButtonDef { id: ButtonId::Mul,      label: "\u{00D7}", col_span: 1, col: 3, row: 1 },

    ButtonDef { id: ButtonId::Digit(4), label: "4", col_span: 1, col: 0, row: 2 },
    ButtonDef { id: ButtonId::Digit(5), label: "5", col_span: 1, col: 1, row: 2 },
    ButtonDef { id: ButtonId::Digit(6), label: "6", col_span: 1, col: 2, row: 2 },
    ButtonDef { id: ButtonId::Sub,      label: "\u{2212}", col_span: 1, col: 3, row: 2 },

    ButtonDef { id: ButtonId::Digit(1), label: "1", col_span: 1, col: 0, row: 3 },
    ButtonDef { id: ButtonId::Digit(2), label: "2", col_span: 1, col: 1, row: 3 },
    ButtonDef { id: ButtonId::Digit(3), label: "3", col_span: 1, col: 2, row: 3 },
    ButtonDef { id: ButtonId::Add,      label: "+", col_span: 1, col: 3, row: 3 },

    ButtonDef { id: ButtonId::Digit(0), label: "0", col_span: 2, col: 0, row: 4 },
    ButtonDef { id: ButtonId::Decimal,  label: ".", col_span: 1, col: 2, row: 4 },
    ButtonDef { id: ButtonId::Equals,   label: "=", col_span: 1, col: 3, row: 4 },
];

const COLS: i32 = 4;
const ROWS: i32 = 5;
const PADDING: i32 = 10;
const GAP: i32 = 6;
pub const DISPLAY_HEIGHT: i32 = 96;

pub fn compute_button_rects(client_w: i32, client_h: i32) -> Vec<Button> {
    let grid_top = DISPLAY_HEIGHT;
    let grid_w = client_w - PADDING * 2;
    let grid_h = client_h - grid_top - PADDING;

    let cell_w = (grid_w - GAP * (COLS - 1)) / COLS;
    let cell_h = (grid_h - GAP * (ROWS - 1)) / ROWS;

    BUTTON_GRID
        .iter()
        .map(|def| {
            let x = PADDING + def.col as i32 * (cell_w + GAP);
            let y = grid_top + def.row as i32 * (cell_h + GAP);
            let w = cell_w * def.col_span as i32 + GAP * (def.col_span as i32 - 1);
            Button {
                id: def.id,
                label: def.label,
                rect: RECT {
                    left: x,
                    top: y,
                    right: x + w,
                    bottom: y + cell_h,
                },
            }
        })
        .collect()
}

pub fn hit_test(buttons: &[Button], x: i32, y: i32) -> Option<usize> {
    buttons.iter().position(|b| {
        x >= b.rect.left && x < b.rect.right && y >= b.rect.top && y < b.rect.bottom
    })
}
