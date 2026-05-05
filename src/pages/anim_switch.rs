use core::time::Duration;

use embassy_time::Instant;
use embedded_graphics::prelude::Point;
use local_static::LocalStatic;
use matrix_gui::{animation::Animations, prelude::*};

// enum RegionID { .. }
// const REGIONID_COUNT: usize
// (RegionID, x, y, width, height)
// Regions: LABEL, SWITCH, EXIT
matrix_gui::free_form_region!(
    RegionId,
    (Background),
    (LABEL, 15, 42, 160, 38),
    (SWITCH, 142, 150, 90, 40),
    (Exit, 15, 150, 73, 40),
);

const WIDGETS_COUNT: usize = REGIONID_COUNT;

static SMARTSTATES: LocalStatic<[RenderState; WIDGETS_COUNT]> = LocalStatic::new();
static ANIMATIONS: LocalStatic<Animations<2>> = LocalStatic::new();

pub struct AnimSwitch<'a> {
    widget_states: WidgetStates<'a>,
    last_down: bool,
    sw_on: bool,
    pages_sw: &'a crate::PageSw,
    anim_manager: AnimManager<'a>,
    sw_anim_id: AnimId,
    lb_anim_id: AnimId,
    last_inst: Instant,
}

impl<'a> AnimSwitch<'a> {
    pub fn new(pages_sw: &'a crate::PageSw) -> Self {
        ANIMATIONS.set(Animations::<2>::new());
        let (animations, anim_status) = ANIMATIONS.get_mut().as_mut();
        let mut anim_manager = AnimManager::new(animations, anim_status);

        let sw_anim = Anim::new(0, 100, Duration::from_millis(250));
        let lb_anim = Anim::new(0, 100, Duration::from_millis(500)).with_easing(Easing::EaseInOut);

        let sw_anim_id = anim_manager.add(sw_anim).expect("Failed to add sw_anim");
        let lb_anim_id = anim_manager.add(lb_anim).expect("Failed to add lb_anim");

        let last_inst = Instant::now();

        Self {
            widget_states: WidgetStates::new_with_anim(SMARTSTATES.get(), anim_status),
            last_down: false,
            sw_on: false,
            pages_sw,
            anim_manager,
            sw_anim_id,
            lb_anim_id,
            last_inst,
        }
    }

    pub fn redraw(&self) {
        self.widget_states.force_redraw_all();
    }

    pub fn update_animations<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let now = Instant::now();
        let delta = now.duration_since(self.last_inst).as_millis();
        if delta > 30 {
            self.last_inst = now;
            if self.anim_manager.tick(Duration::from_millis(delta)) {
                self.widget_states.force_redraw(LABEL.id());
                self.update(true, Point::zero(), display);
            }
        }
    }

    pub fn update<D>(&mut self, tp_down: bool, location: Point, display: &mut D)
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let mut ui = Ui::new_fullscreen(display, &self.widget_states, crate::example_style());
        super::ui_interact(self.last_down, tp_down, location, &mut ui);
        self.last_down = tp_down;

        ui.add(Background::new(RegionId::Background));

        ui.lazy_draw(LABEL.id(), |ui| {
            let status = ui.take_anim_status(self.lb_anim_id).unwrap_or_default();
            let offset = if let Some(offset) = status { offset } else { 0 };

            let lb_area = if self.sw_on {
                LABEL.move_by(offset as i16, 0)
            } else {
                LABEL.move_by(100 - offset as i16, 0)
            };
            let clr_rect = LABEL
                .area()
                .resized(offset as u16, LABEL.height() as u16, AnchorPoint::TopLeft)
                .rectangle();
            ui.clear_area(&clr_rect).ok();
            ui.add(Label::new(&lb_area, "Hello animations"))
        });

        if ui.add(Button::new(EXIT, "Exit")).is_clicked() {
            self.pages_sw.signal(crate::Pages::Home);
        }

        let resp = ui.add(Switch::new(SWITCH, &mut self.sw_on, self.sw_anim_id));
        if resp.is_clicked() {
            log::info!("Switch clicked: {}", self.sw_on);
            self.anim_manager.play(self.sw_anim_id);
            self.anim_manager.play(self.lb_anim_id);
            ui.force_redraw(LABEL.id());
        }
    }
}

pub struct Switch<'a, ID> {
    region: &'a Region<ID>,
    on: &'a mut bool,
    anim_id: AnimId,
}

impl<'a, ID: WidgetId> Switch<'a, ID> {
    pub fn new(region: &'a Region<ID>, on: &'a mut bool, anim_id: AnimId) -> Self {
        Self {
            region,
            on,
            anim_id,
        }
    }
}

impl<'a, DRAW, ID: WidgetId> Widget<DRAW, Rgb565> for Switch<'a, ID>
where
    DRAW: DrawTarget<Color = Rgb565>,
{
    fn draw(&mut self, ui: &mut Ui<DRAW, Rgb565>) -> GuiResult<Response> {
        let widget_id = self.region.id();
        let mut interaction = ui.check_interact(self.region);
        let prevstate = ui.get_widget_state(widget_id)?.status();
        let move_state = ui.take_anim_status(self.anim_id)?;

        let next_state;
        match interaction {
            Interaction::None => {
                if move_state.is_none() {
                    next_state = RenderStatus::Rendered;
                } else {
                    next_state = RenderStatus::NeedsRedraw;
                }
            }
            Interaction::Pressed(_) | Interaction::Drag(_) => {
                next_state = RenderStatus::Pressed;
            }
            Interaction::Release(pos) | Interaction::Clicked(pos) => {
                *self.on = !*self.on;
                next_state = RenderStatus::Released;
                interaction = Interaction::Clicked(pos);
            }
            _ => {
                next_state = RenderStatus::Unknown;
            }
        }

        if next_state != RenderStatus::NeedsRedraw && next_state == prevstate {
            return Ok(interaction.into());
        }
        ui.get_widget_state(widget_id)?.set_status(next_state);

        let area = self.region.rectangle();

        // let move_state = move_state.unwrap_or_default();
        let height = area.size.height;
        let rounded_rect = matrix_utils::make_rounded_rect(&area, height / 2);
        let fill_col = matrix_utils::select(*self.on, rgb565!(0x2196F3), rgb565!(0xc0c0c0));
        let mut rect_style: PrimitiveStyle<Rgb565> = PrimitiveStyle::with_fill(fill_col);

        let delta_width = (area.size.width - height) as i32;
        let left_center = area.center() - Point::new(delta_width / 2, 0);
        let right_center = area.center() + Point::new(delta_width / 2, 0);

        let center = if let Some(offset) = move_state {
            if *self.on {
                left_center + Point::new(offset * delta_width / 100, 0)
            } else {
                right_center - Point::new(offset * delta_width / 100, 0)
            }
        } else {
            if !*self.on { left_center } else { right_center }
        };

        let lever_rect = Rectangle::with_center(center, Size::new_equal(height - 2));
        let lever = matrix_utils::make_rounded_rect(&lever_rect, height / 2);

        ui.draw(&rounded_rect.into_styled(rect_style)).ok();
        rect_style.fill_color = Some(Rgb565::WHITE);
        ui.draw(&lever.into_styled(rect_style)).ok();

        Ok(interaction.into())
    }
}
