use core::time::Duration;

use embedded_graphics::prelude::Point;
use matrix_gui::{animation::Animations, prelude::*};
use multi_mono_font::MultiMonoFont;
use static_cell::StaticCell;

// enum RegionID { .. }
// const REGIONID_COUNT: usize
// (RegionID, x, y, width, height)
// Regions: SECOND1, SECOND10, MINUTE1, MINUTE10, HOUR1, HOUR10, EXIT, TEST, SEP1, SEP0
matrix_gui::free_form_region!(
    RegionId,
    (Background),
    (Second1, 189, 85, 24, 48),
    (Second10, 165, 85, 24, 48),
    (Minute1, 117, 85, 24, 48),
    (Minute10, 93, 85, 24, 48),
    (Hour1, 45, 85, 24, 48),
    (Hour10, 21, 85, 24, 48),
    (Quit, 200, 197, 58, 23),
    (Sep1, 69, 85, 24, 48),
    (Sep0, 141, 85, 24, 48),
);

const NUM_FONT: MultiMonoFont<'static> = multi_mono_font_glyph::generate_font!(
    text = "0123456789:",
    font = "simsun.ttc",
    font_size = 40,
    char_width = 24,
    char_height = 40,
    x_offset = 0,
    y_offset = 0,
    baseline = 40,
    threshold = 30,
);

const ANIMATIONS_COUNT: usize = 6;
const WIDGETS_COUNT: usize = REGIONID_COUNT;

struct HMS {
    sec1: u8,
    sec10: u8,
    min1: u8,
    min10: u8,
    hour1: u8,
    hour10: u8,
}

impl HMS {
    fn from(utc: u32) -> Self {
        let hour = utc / 3600;
        let temp = utc % 3600;
        let min = temp / 60;
        let sec = temp % 60;

        let (sec, min, hour) = (sec as u8, min as u8, hour as u8);
        Self {
            sec1: sec % 10,
            sec10: sec / 10,
            min1: min % 10,
            min10: min / 10,
            hour1: hour % 10,
            hour10: hour / 10,
        }
    }
}

pub struct Clock<'a> {
    widget_states: WidgetStates<'a>,
    last_down: bool,
    pages_sw: &'a crate::PageSw,
    anim_manager: AnimManager<'a>,
    anim_id_second_x1: AnimId,
    anim_id_second_x10: AnimId,
    anim_id_minute_x1: AnimId,
    anim_id_minute_x10: AnimId,
    anim_id_hour_x1: AnimId,
    anim_id_hour_x10: AnimId,
    utc: u32,
    hms_last: HMS,
    hms_curr: HMS,
}

impl<'a> Clock<'a> {
    pub fn new(pages_sw: &'a crate::PageSw) -> Self {
        let (animations, anim_status) = {
            static ANIMATIONS: StaticCell<Animations<ANIMATIONS_COUNT>> = StaticCell::new();
            ANIMATIONS
                .init(Animations::<ANIMATIONS_COUNT>::new())
                .as_mut()
        };
        let mut anim_manager = AnimManager::new(animations, anim_status);

        let anim_second_x1 = Anim::new(0, 100, Duration::from_millis(500));
        let anim_second_x10 = Anim::new(0, 100, Duration::from_millis(500));
        let anim_minute_x1 = Anim::new(0, 100, Duration::from_millis(500));
        let anim_minute_x10 = Anim::new(0, 100, Duration::from_millis(500));
        let anim_hour_x1 = Anim::new(0, 100, Duration::from_millis(500));
        let anim_hour_x10 = Anim::new(0, 100, Duration::from_millis(500));

        let anim_id_second_x1 = anim_manager
            .add(anim_second_x1)
            .expect("Failed to add anim_second_x1");
        let anim_id_second_x10 = anim_manager
            .add(anim_second_x10)
            .expect("Failed to add anim_second_x10");
        let anim_id_minute_x1 = anim_manager
            .add(anim_minute_x1)
            .expect("Failed to add anim_minute_x1");
        let anim_id_minute_x10 = anim_manager
            .add(anim_minute_x10)
            .expect("Failed to add anim_minute_x10");
        let anim_id_hour_x1 = anim_manager
            .add(anim_hour_x1)
            .expect("Failed to add anim_hour_x1");
        let anim_id_hour_x10 = anim_manager
            .add(anim_hour_x10)
            .expect("Failed to add anim_hour_x10");

        let utc = 9 * 3600 + 59 * 60 + 55;

        let states = {
            static SMARTSTATES: StaticCell<[RenderState; WIDGETS_COUNT]> = StaticCell::new();
            SMARTSTATES.init(RenderState::new_array())
        };

        Self {
            widget_states: WidgetStates::new_with_anim(states, anim_status),
            last_down: false,
            pages_sw,
            anim_manager,
            anim_id_second_x1,
            anim_id_second_x10,
            anim_id_minute_x1,
            anim_id_minute_x10,
            anim_id_hour_x1,
            anim_id_hour_x10,
            utc,
            hms_last: HMS::from(utc),
            hms_curr: HMS::from(utc),
        }
    }

    pub fn add_one_second(&mut self) {
        self.hms_last = HMS::from(self.utc);
        self.utc += 1;
        self.hms_curr = HMS::from(self.utc);
        if self.hms_last.sec1 != self.hms_curr.sec1 {
            self.widget_states.force_redraw(SECOND1.id());
            self.anim_manager.play(self.anim_id_second_x1);
        }
        if self.hms_last.sec10 != self.hms_curr.sec10 {
            self.widget_states.force_redraw(SECOND10.id());
            self.anim_manager.play(self.anim_id_second_x10);
        }
        if self.hms_last.min1 != self.hms_curr.min1 {
            self.widget_states.force_redraw(MINUTE1.id());
            self.anim_manager.play(self.anim_id_minute_x1);
        }
        if self.hms_last.min10 != self.hms_curr.min10 {
            self.widget_states.force_redraw(MINUTE10.id());
            self.anim_manager.play(self.anim_id_minute_x10);
        }
        if self.hms_last.hour1 != self.hms_curr.hour1 {
            self.widget_states.force_redraw(HOUR1.id());
            self.anim_manager.play(self.anim_id_hour_x1);
        }
        if self.hms_last.hour10 != self.hms_curr.hour10 {
            self.widget_states.force_redraw(HOUR10.id());
            self.anim_manager.play(self.anim_id_hour_x10);
        }
    }

    pub fn redraw(&self) {
        self.widget_states.force_redraw_all();
    }

    pub fn update_animations<D>(&mut self, elapsed: Duration, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
    {
        if self.anim_manager.tick(elapsed) {
            self.update(self.last_down, Point::new(-1, -1), display);
        }
    }

    pub fn update<D>(&mut self, tp_down: bool, location: Point, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let anim_update = location == Point::new(-1, -1);
        let mut ui = Ui::new_fullscreen(display, &self.widget_states, crate::example_style());
        if !anim_update {
            super::ui_interact(self.last_down, tp_down, location, &mut ui);
            self.last_down = tp_down;
        }

        ui.add(Background::new(RegionId::Background));
        ui.add(Label::new(SEP1, ":").with_font(&[&NUM_FONT]));
        ui.add(Label::new(SEP0, ":").with_font(&[&NUM_FONT]));

        ui.add(RollingDigit::new(
            SECOND1,
            self.hms_last.sec1,
            self.hms_curr.sec1,
            self.anim_id_second_x1,
        ));

        ui.add(RollingDigit::new(
            SECOND10,
            self.hms_last.sec10,
            self.hms_curr.sec10,
            self.anim_id_second_x10,
        ));

        ui.add(RollingDigit::new(
            MINUTE1,
            self.hms_last.min1,
            self.hms_curr.min1,
            self.anim_id_minute_x1,
        ));

        ui.add(RollingDigit::new(
            MINUTE10,
            self.hms_last.min10,
            self.hms_curr.min10,
            self.anim_id_minute_x10,
        ));

        ui.add(RollingDigit::new(
            HOUR1,
            self.hms_last.hour1,
            self.hms_curr.hour1,
            self.anim_id_hour_x1,
        ));

        ui.add(RollingDigit::new(
            HOUR10,
            self.hms_last.hour10,
            self.hms_curr.hour10,
            self.anim_id_hour_x10,
        ));

        if !anim_update {
            if ui.add(Button::new(QUIT, "Quit")).is_clicked() {
                log::info!("Quit clicked");
                self.pages_sw.signal(crate::Pages::Home);
            }
        }
    }
}

pub struct RollingDigit<'a, ID> {
    region: &'a Region<ID>,
    curr: u8,
    next: u8,
    anim_id: AnimId,
}

impl<'a, ID: WidgetId> RollingDigit<'a, ID> {
    pub fn new(region: &'a Region<ID>, curr: u8, next: u8, anim_id: AnimId) -> Self {
        Self {
            region,
            curr,
            next,
            anim_id,
        }
    }
}

const NUM_MAP: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

impl<'a, DRAW, ID: WidgetId> Widget<DRAW, Rgb565> for RollingDigit<'a, ID>
where
    DRAW: DrawTarget<Color = Rgb565>,
{
    fn draw(&mut self, ui: &mut Ui<DRAW, Rgb565>) -> GuiResult<Response> {
        let widget_id = self.region.id();
        let render_state = ui.get_widget_state(widget_id)?;
        let move_state = ui.take_anim_status(self.anim_id)?;

        if move_state.is_none() && render_state.compare(RenderStatus::Rendered) {
            return Ok(Response::Idle);
        }

        render_state.set_status(RenderStatus::Rendered);

        let mut area = self.region.rectangle();
        ui.clear_area(&area)?;
        ui.set_clipped_area(Some(area));
        if let Some(offset) = move_state {
            area.top_left.y -= offset * area.size.height as i32 / 100;
        }
        let font = &[&NUM_FONT]; //ui.style().default_font;
        let color = ui.style().text_color;
        let curr_text = NUM_MAP.get(self.curr as usize).unwrap_or(&"0");
        let mut text = matrix_utils::make_text(curr_text, font, color);
        matrix_utils::text_align_translate(&mut text, &area, HorizontalAlign::Center);
        ui.draw(&text)?;

        area.top_left.y += area.size.height as i32;
        let next_text = NUM_MAP.get(self.next as usize).unwrap_or(&"0");
        let mut next_text = matrix_utils::make_text(next_text, font, color);
        matrix_utils::text_align_translate(&mut next_text, &area, HorizontalAlign::Center);
        ui.draw(&next_text)?;
        ui.set_clipped_area(None);

        Ok(Response::Idle)
    }
}
