use embedded_graphics::prelude::Point;
use local_static::LocalStatic;
use matrix_gui::prelude::*;
// enum RegionId { .. }
// const REGIONID_COUNT: usize
// (RegionID, x, y, width, height)
// Regions: MSG_BOX, MSG_BOX_OK
matrix_gui::free_form_region!(
    RegionId,
    (Background),
    (MSG_BOX, 20, 40, 220, 160),
    (MSG_BOX_OK, 140, 150, 82, 40),
);

// enum Btn4X6 { .. }
// const BTN_4X6_COUNT: usize
// const BTN_4X6_AREA: Rectangle
// const RECT_1_BTN_4X6_GLRMGLRM: [Region<Rect1>; RECT_1_COUNT]
#[rustfmt::skip]
matrix_gui::grid_layout_row_major_with_start! (
    BTN_4X6,
    (REGIONID_COUNT),
    (10, 20, 260, 200),
    (4, 6, 2),
    [
        CELL_0, CELL_1, CELL_2, CELL_3, CELL_4, CELL_5,
        CELL_6, CELL_7, CELL_8, CELL_9, CELL_10, CELL_11,
        CELL_12, CELL_13, CELL_14, CELL_15, CELL_16, CELL_17,
        CELL_18, CELL_19, CELL_20, CELL_21, CELL_22, CELL_23,
    ]
);

const WIDGETS_COUNT: usize = BTN_4X6_COUNT + REGIONID_COUNT;

static SMARTSTATES: LocalStatic<[RenderState; WIDGETS_COUNT]> = LocalStatic::new();

pub struct MsgBox<'a> {
    widget_states: WidgetStates<'a>,
    last_down: bool,
    show: bool,
    pages_sw: &'a crate::PageSw,
}

impl<'a> MsgBox<'a> {
    pub fn new(pages_sw: &'a crate::PageSw) -> Self {
        Self {
            widget_states: WidgetStates::new(SMARTSTATES.get()),
            last_down: false,
            show: false,
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
            super::ui_interact(self.last_down, tp_down, location, &mut ui);
            self.last_down = tp_down;

            ui.add(Background::new(RegionId::Background));

            if !self.show {
                if ui.add(Button::new(&BTN_4X6_GLRM[0], "Show")).is_clicked() {
                    log::info!("BTN0 clicked");
                    self.show = true;
                }
                const BTN_LB_LIST: &[&str] = &[
                    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                    "15", "16", "17", "18", "19", "20", "21", "22", "23",
                ];
                for (idx, btn) in (BTN_4X6_GLRM[1..]).iter().enumerate() {
                    ui.add(Button::new(btn, BTN_LB_LIST[idx]));
                }
            } else {
                let response = ui.add(
                    MessageBox::new(MSG_BOX, "This is title", "Hello World\nHello Matrix GUI!")
                        .with_ok_btn(MSG_BOX_OK, "OoooK"),
                );
                if response.is_clicked() {
                    log::info!("OK msg clicked");
                    self.show = false;
                    self.pages_sw.signal(crate::Pages::Home);
                    break;
                }
            }
        }
    }
}
