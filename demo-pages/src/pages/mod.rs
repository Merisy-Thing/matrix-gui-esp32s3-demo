use local_static::LocalStatic;
use matrix_gui::prelude::*;

pub mod anim_switch;
pub mod basic_example;
pub mod calculator;
pub mod msg_box;

pub use anim_switch::*;
pub use basic_example::*;
pub use calculator::*;
pub use msg_box::*;

// enum RegionId { .. }
// const REGIONID_COUNT: usize
matrix_gui::free_form_region!(RegionId, (Background), (TITLE, 38, 5, 214, 20),);

// enum Home { .. }
// const HOME_COUNT: usize
// const HOME_AREA: Rectangle
// const HOME_GLRM: [Region<Home>; HOME_COUNT]
#[rustfmt::skip]
matrix_gui::grid_layout_row_major_with_start! (
    Home,
    REGIONID_COUNT,
    (15, 38, 250, 195),
    (4, 3, 5),
    [
        Basic, MsgBox, Calculator, AnimSwitch,
    ]
);

const WIDGETS_COUNT: usize = REGIONID_COUNT + HOME_COUNT;

static SMARTSTATES: LocalStatic<[RenderState; WIDGETS_COUNT]> = LocalStatic::new();

pub struct HomePage<'a> {
    widget_states: WidgetStates<'a>,
    last_down: bool,
    pages_sw: &'a crate::PageSw,
}

impl<'a> HomePage<'a> {
    pub fn new(pages_sw: &'a crate::PageSw) -> Self {
        Self {
            widget_states: WidgetStates::new(SMARTSTATES.get()),
            last_down: false,
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
        for _ in 0..2 {
            let mut ui = Ui::new_fullscreen(display, &self.widget_states, crate::example_style());
            ui_interact(self.last_down, tp_down, location, &mut ui);
            self.last_down = tp_down;

            ui.add(Background::new(RegionId::Background));

            ui.add(Label::new(TITLE, "Matrix GUI 示例").with_align(HorizontalAlign::Center));

            if ui.add(Button::new(BASIC, "Basic")).is_clicked() {
                self.pages_sw.signal(crate::Pages::Basic);
            }
            if ui.add(Button::new(MSGBOX, "MsgBox")).is_clicked() {
                self.pages_sw.signal(crate::Pages::MsgBox);
            }
            if ui.add(Button::new(CALCULATOR, "Calc")).is_clicked() {
                self.pages_sw.signal(crate::Pages::Calculator);
            }
            if ui.add(Button::new(ANIMSWITCH, "Anim SW")).is_clicked() {
                self.pages_sw.signal(crate::Pages::AnimSwitch);
            }
            break;
        }
    }
}

pub fn ui_interact<'a, COL, DRAW>(
    last_down: bool,
    tp_down: bool,
    location: Point,
    ui: &mut Ui<'a, COL, DRAW>,
) where
    COL: DrawTarget<Color = DRAW>,
    DRAW: PixelColor,
{
    match (last_down, tp_down, location) {
        (false, true, loc) => {
            ui.interact(Interaction::Pressed(loc));
        }
        (true, true, loc) => {
            ui.interact(Interaction::Drag(loc));
        }
        (true, false, loc) => {
            ui.interact(Interaction::Release(loc));
        }
        (false, false, _) => {}
    }
}
