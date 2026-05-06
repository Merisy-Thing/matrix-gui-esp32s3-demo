use core::fmt::Write;
use embedded_graphics::prelude::Point;
use heapless::{String, Vec};
use local_static::LocalStatic;
use matrix_gui::prelude::*;
#[allow(unused_imports)]
use num_traits::float::FloatCore;

// enum RegionId { .. }
// const REGIONID_COUNT: usize
matrix_gui::free_form_region!(
    RegionId,
    (Background),
    (Expression, 5, 6, 128, 43),
    (AC, 141, 6, 130, 43),
);

// enum Calc { .. }
// const CALC_COUNT: usize
// const CALC_AREA: Rectangle
// const CALC_GLRM: [Region<Calc>; CALC_COUNT]
#[rustfmt::skip]
matrix_gui::grid_layout_row_major_with_start! (
    Calc,
	REGIONID_COUNT,
    (5, 55, 270, 182),
    (4, 4, 5),
    [
        Num7, Num8, Num9, Div,
        Num4, Num5, Num6, Mul,
        Num1, Num2, Num3, Sub,
        Num0, Dot, Equal, Add
    ]
);

const WIDGETS_COUNT: usize = CALC_COUNT + REGIONID_COUNT;

static SMARTSTATES: LocalStatic<[RenderState; WIDGETS_COUNT]> = LocalStatic::new();

pub struct Calculator<'a> {
    widget_states: WidgetStates<'a>,
    last_down: bool,
    expression: String<32>,
    pages_sw: &'a crate::PageSw,
}

impl<'a> Calculator<'a> {
    pub fn new(pages_sw: &'a crate::PageSw) -> Self {
        Self {
            widget_states: WidgetStates::new(SMARTSTATES.get()),
            last_down: false,
            expression: String::new(),
            pages_sw,
        }
    }

    pub fn redraw(&self) {
        self.widget_states.force_redraw_all();
    }

    pub fn update<D>(&mut self, tp_down: bool, location: Point, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
    {
        loop {
            let mut ui = Ui::new_fullscreen(display, &self.widget_states, crate::example_style());
            super::ui_interact(self.last_down, tp_down, location, &mut ui);
            self.last_down = tp_down;

            ui.add(Background::new(RegionId::Background));

            let mut input = None;
            ui.add(Label::new(EXPRESSION, &self.expression).with_align(HorizontalAlign::Right));
            if ui.add(Button::new(AC, "AC")).is_clicked() {
                log::info!("Btn1 clicked");
                input = Some("AC");
            }

            const BTN_LB_LIST: &[&str] = &[
                "7", "8", "9", "/", "4", "5", "6", "X", "1", "2", "3", "-", "0", ".", "=", "+",
            ];

            for (idx, btn) in CALC_GLRM.iter().enumerate() {
                if ui.add(Button::new(btn, BTN_LB_LIST[idx])).is_clicked() {
                    input = Some(BTN_LB_LIST[idx]);
                }
            }

            if let Some(input) = input {
                self.user_input(input);
                self.widget_states.force_redraw(EXPRESSION.id());
                continue;
            }

            break;
        }
    }

    fn user_input(&mut self, input: &str) {
        match input {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                if self.expression.len() < 31 {
                    let _ = self.expression.push_str(input);
                }
            }
            "." => {
                if self.expression.ends_with('.') {
                    self.pages_sw.signal(crate::Pages::Home);
                } else if self.expression.len() < 31 {
                    let expr_str = self.expression.as_str();
                    if expr_str.is_empty() {
                        let _ = self.expression.push_str("0.");
                    } else {
                        let last_part = expr_str
                            .split(|c| c == '+' || c == '-' || c == 'X' || c == '/')
                            .last()
                            .unwrap_or("");
                        if !last_part.contains('.') {
                            let _ = self.expression.push('.');
                        }
                    }
                }
            }
            "+" | "-" | "X" | "/" => {
                if !self.expression.is_empty() {
                    let last_char = self.expression.chars().last().unwrap();
                    if !matches!(last_char, '+' | '-' | 'X' | '/') {
                        if self.expression.len() < 31 {
                            let _ = self.expression.push_str(input);
                        }
                    }
                }
            }
            "=" => {
                if !self.expression.is_empty() {
                    let result = self.evaluate_expression();
                    match result {
                        Ok(value) => {
                            self.expression.clear();
                            let _ = self.expression.push_str(value.as_str());
                        }
                        Err(_) => {
                            self.expression.clear();
                            let _ = self.expression.push_str("Error");
                        }
                    }
                }
            }
            "AC" => {
                self.expression.clear();
            }
            _ => {}
        }
    }

    fn evaluate_expression(&self) -> Result<String<32>, ()> {
        let expr = self.expression.as_str();

        let mut numbers: Vec<f32, 16> = Vec::new();
        let mut operators: Vec<char, 16> = Vec::new();

        let mut current_num: String<16> = String::new();

        for ch in expr.chars() {
            match ch {
                '0'..='9' | '.' => {
                    if current_num.push(ch).is_err() {
                        return Err(());
                    }
                }
                '+' | '-' | 'X' | '/' => {
                    if !current_num.is_empty() {
                        match current_num.parse::<f32>() {
                            Ok(num) => {
                                if numbers.push(num).is_err() {
                                    return Err(());
                                }
                            }
                            Err(_) => return Err(()),
                        }
                        current_num.clear();
                    }
                    if operators.push(ch).is_err() {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }

        if !current_num.is_empty() {
            match current_num.parse::<f32>() {
                Ok(num) => {
                    if numbers.push(num).is_err() {
                        return Err(());
                    }
                }
                Err(_) => return Err(()),
            }
        }

        if numbers.is_empty() || numbers.len() != operators.len() + 1 {
            return Err(());
        }

        let mut i = 0;
        while i < operators.len() {
            if operators[i] == 'X' || operators[i] == '/' {
                let left = numbers[i];
                let right = numbers[i + 1];
                let result = if operators[i] == 'X' {
                    left * right
                } else {
                    if right == 0.0 {
                        return Err(());
                    }
                    left / right
                };
                numbers[i] = result;
                numbers.remove(i + 1);
                operators.remove(i);
            } else {
                i += 1;
            }
        }

        let mut result = numbers[0];
        for (i, &op) in operators.iter().enumerate() {
            match op {
                '+' => result += numbers[i + 1],
                '-' => result -= numbers[i + 1],
                _ => return Err(()),
            }
        }

        let formatted = if result.fract() == 0.0 {
            let mut s: String<32> = String::new();
            let _ = write!(s, "{}", result as i32);
            s
        } else {
            let mut s: String<32> = String::new();
            let _ = write!(s, "{:.6}", result);
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
            s
        };

        if formatted.len() > 31 {
            let mut overflow: String<32> = String::new();
            let _ = overflow.push_str("Overflow");
            Ok(overflow)
        } else {
            Ok(formatted)
        }
    }
}
