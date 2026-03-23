#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

pub fn op_symbol(op: Op) -> &'static str {
    match op {
        Op::Add => "+",
        Op::Sub => "−",
        Op::Mul => "×",
        Op::Div => "÷",
    }
}

pub struct Calculator {
    pub display: String,
    /// What the formula bar shows. Updated explicitly on operator/equals/clear.
    pub expression: String,
    accumulator: Option<f64>,
    pending_op: Option<Op>,
    /// Next digit input clears the display and starts fresh.
    awaiting_operand: bool,
    /// Last action was '='; digit/decimal after this starts a brand-new equation.
    just_evaluated: bool,
    last_rhs: Option<f64>,
    pub is_error: bool,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            display: String::from("0"),
            expression: String::new(),
            accumulator: None,
            pending_op: None,
            awaiting_operand: false,
            just_evaluated: false,
            last_rhs: None,
            is_error: false,
        }
    }

    pub fn digit(&mut self, d: u8) {
        if self.is_error {
            return;
        }
        if self.just_evaluated {
            // Starting a brand-new equation after '=' — wipe everything
            *self = Self::new();
            self.display = d.to_string();
            return;
        }
        if self.awaiting_operand {
            self.display = d.to_string();
            self.awaiting_operand = false;
        } else {
            if self.display.len() >= 15 {
                return;
            }
            if self.display == "0" {
                self.display = d.to_string();
            } else {
                self.display.push(char::from_digit(d as u32, 10).unwrap());
            }
        }
    }

    pub fn decimal(&mut self) {
        if self.is_error {
            return;
        }
        if self.just_evaluated {
            *self = Self::new();
            self.display = String::from("0.");
            return;
        }
        if self.awaiting_operand {
            self.display = String::from("0.");
            self.awaiting_operand = false;
        } else if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    pub fn operator(&mut self, op: Op) {
        if self.is_error {
            return;
        }
        let val = self.parse_display();

        if self.pending_op.is_some() && !self.awaiting_operand {
            // Chain: evaluate the pending operation first
            let result = evaluate(self.accumulator.unwrap_or(val), self.pending_op.unwrap(), val);
            match result {
                Ok(v) => {
                    self.accumulator = Some(v);
                    self.display = format_number(v);
                }
                Err(_) => {
                    self.set_error();
                    return;
                }
            }
        } else {
            self.accumulator = Some(val);
        }

        self.pending_op = Some(op);
        self.awaiting_operand = true;
        self.just_evaluated = false;

        // Formula bar: "acc op"
        let acc = self.accumulator.unwrap();
        self.expression = format!("{} {}", format_number(acc), op_symbol(op));
    }

    pub fn equals(&mut self) {
        if self.is_error || self.pending_op.is_none() {
            return;
        }
        let op = self.pending_op.unwrap();
        let lhs = self.accumulator.unwrap_or_else(|| self.parse_display());
        let rhs = if self.just_evaluated {
            self.last_rhs.unwrap_or_else(|| self.parse_display())
        } else {
            self.parse_display()
        };
        self.last_rhs = Some(rhs);

        // Capture full expression before evaluating
        let full_expr = format!(
            "{} {} {} =",
            format_number(lhs),
            op_symbol(op),
            format_number(rhs)
        );

        match evaluate(lhs, op, rhs) {
            Ok(v) => {
                self.display = format_number(v);
                self.accumulator = Some(v);
                self.awaiting_operand = true;
                self.just_evaluated = true;
                self.expression = full_expr;
            }
            Err(_) => {
                self.set_error();
            }
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn clear_entry(&mut self) {
        if self.just_evaluated {
            // CE after evaluation is effectively the same as C
            *self = Self::new();
            return;
        }
        self.display = String::from("0");
        self.is_error = false;
        self.awaiting_operand = false;
        // expression intentionally kept — user may be correcting the RHS of a pending op
    }

    pub fn backspace(&mut self) {
        if self.awaiting_operand || self.is_error {
            return;
        }
        let neg_only = self.display == "-";
        if self.display.len() <= 1 || neg_only {
            self.display = String::from("0");
        } else {
            self.display.pop();
        }
    }

    fn parse_display(&self) -> f64 {
        self.display.parse().unwrap_or(0.0)
    }

    fn set_error(&mut self) {
        self.display = String::from("Cannot divide by zero");
        self.expression = String::new();
        self.is_error = true;
        self.accumulator = None;
        self.pending_op = None;
        self.awaiting_operand = false;
        self.just_evaluated = false;
    }
}

fn evaluate(lhs: f64, op: Op, rhs: f64) -> Result<f64, ()> {
    match op {
        Op::Add => Ok(lhs + rhs),
        Op::Sub => Ok(lhs - rhs),
        Op::Mul => Ok(lhs * rhs),
        Op::Div => {
            if rhs == 0.0 {
                Err(())
            } else {
                Ok(lhs / rhs)
            }
        }
    }
}

fn format_number(v: f64) -> String {
    if v.is_nan() || v.is_infinite() {
        return String::from("Error");
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.10}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
