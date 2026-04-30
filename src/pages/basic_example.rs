use core::fmt::Write;
use embedded_graphics::{image::ImageRaw, prelude::Point};
use local_static::LocalStatic;
use matrix_gui::prelude::*;
use multi_mono_font::MonoImage;

pub const WIDGETS_COUNT: usize = REGIONID_COUNT;

#[derive(Debug, Clone, Copy, PartialEq)]
enum RadioGroup {
    None,
    Btn1,
    Btn2,
    Btn3,
}
const RADIOBUTTON_IDS: &[RegionId] = &[RADIOBUTTON1.id(), RADIOBUTTON2.id(), RADIOBUTTON3.id()];

const IMAGE_RAW: ImageRaw<BinaryColor> =
    ImageRaw::<BinaryColor>::new(include_bytes!("../../assets/rust_64x64.bin"), 64);
const MONO_IMAGE: MonoImage<Rgb565> = MonoImage::<Rgb565>::new(&IMAGE_RAW, rgb565!(0xFF0000));
matrix_gui::i18n_string!(TIP_ON, "打开", "ON");
matrix_gui::i18n_string!(TIP_OFF, "关闭", "OFF");
matrix_gui::i18n_toggle_type!(TipOnOff, TIP_ON, TIP_OFF);

// enum RegionID { .. }
// const REGIONID_COUNT: usize
// (RegionID, x, y, width, height)
// Regions: TITLE, LABEL1, LABEL2, BUTTON1, BUTTON2, BUTTON3, BAR, LINE_V1, LINE_V2, LINE_H1,
// SLIDER, CHECKBOX_1, CHECKBOX_2, RADIOBUTTON1, RADIOBUTTON2, RADIOBUTTON3, IMAGE
matrix_gui::free_form_region!(
    RegionId,
    (Background),
    (TITLE, 38, 3, 214, 24),
    (LABEL1, 9, 29, 130, 20),
    (LABEL2, 158, 29, 116, 18),
    (BUTTON1, 16, 77, 61, 24),
    (BUTTON2, 99, 77, 79, 24),
    (BUTTON3, 191, 77, 85, 24),
    (BAR, 20, 110, 249, 14),
    (LINE_V1, 10, 131, 12, 100),
    (LINE_V2, 258, 132, 14, 91),
    (LINE_H1, 37, 128, 202, 11),
    (SLIDER, 41, 209, 202, 22),
    (CHECKBOX_1, 36, 146, 130, 26),
    (CHECKBOX_2, 36, 177, 130, 25),
    (RADIOBUTTON1, 12, 53, 74, 20),
    (RADIOBUTTON2, 93, 53, 88, 20),
    (RADIOBUTTON3, 188, 53, 85, 20),
    (IMAGE, 176, 146, 64, 64),
);

static SMARTSTATES: LocalStatic<[RenderState; WIDGETS_COUNT]> = LocalStatic::new();

pub struct BasicExample<'a> {
    widget_states: WidgetStates<'a>,
    last_down: bool,
    slider_val: i16,
    checkbox1: bool,
    checkbox2: bool,
    radio: RadioGroup,
    label1: heapless::String<32>,
    pages_sw: &'a crate::PageSw,
}

impl<'a> BasicExample<'a> {
    pub fn new(pages_sw: &'a crate::PageSw) -> Self {
        let label1 = {
            let mut lb_str = heapless::String::<32>::new();
            let tip: &str = &TipOnOff(false);
            write!(&mut lb_str, "{}", tip).unwrap();
            lb_str
        };

        Self {
            widget_states: WidgetStates::new(SMARTSTATES.get()),
            last_down: false,
            slider_val: 0,
            checkbox1: false,
            checkbox2: false,
            radio: RadioGroup::None,
            label1,
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

            match (self.last_down, tp_down, location) {
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
            self.last_down = tp_down;

            ui.add(Background::new(RegionId::Background));

            if self.widget_states.should_redraw_multi(&[TITLE.id()]) {
                log::info!("Static 1 redraw needed");
                ui.add(Label::new(TITLE, "Matrix GUI 示例").with_align(HorizontalAlign::Center));
            }
            ui.add(Label::new(LABEL1, &self.label1));

            ui.add(StaticImage::new(IMAGE, &MONO_IMAGE));

            ui.lazy_draw(LABEL2.id(), |lazy_ui| {
                log::info!("Label2 redraw");
                let mut lb_str: heapless::String<32> = heapless::String::new();

                write!(&mut lb_str, "Label2 {}", self.slider_val).unwrap();
                lazy_ui.add(Label::new(LABEL2, &lb_str).with_align(HorizontalAlign::Right))
            });

            if ui.add(Button::new(BUTTON1, "Btn1")).is_clicked() {
                log::info!("Btn1 clicked");
                self.label1.clear();
                write!(&mut self.label1, "Btn1").unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                continue;
            }
            if ui.add(Button::new(BUTTON2, "Button2")).is_clicked() {
                log::info!("Btn2 clicked");
                self.label1.clear();
                write!(&mut self.label1, "Btn2").unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                continue;
            }
            if ui.add(Button::new(BUTTON3, "Button3")).is_clicked() {
                log::info!("Btn3 clicked");
                self.label1.clear();
                write!(&mut self.label1, "Btn3").unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                self.pages_sw.signal(crate::Pages::MsgBox);
                continue;
            }

            if self
                .widget_states
                .should_redraw_multi(&[LINE_V1.id(), LINE_V2.id(), LINE_H1.id()])
            {
                log::info!("Lines redraw needed");
                ui.add(StaticLine::new(LINE_V1, &OriVertical));
                ui.add(StaticLine::new(LINE_V2, &OriVertical));
                ui.add(StaticLine::new(LINE_H1, &OriHorizontal));
            }

            if self.widget_states.should_redraw_multi(&[BAR.id()]) {
                log::info!("Bar redraw needed");
                ui.add(Bar::new(BAR, -100, 100, self.slider_val).with_border_color(Rgb565::BLACK));
            }

            if ui
                .add(
                    Slider::new(SLIDER, &mut self.slider_val, -100..=100)
                        .label("Fancy Slider")
                        .step_size(5),
                )
                .is_value_changed()
            {
                log::info!("Slider value: {}", self.slider_val);
                self.widget_states
                    .force_redraw_multi(&[BAR.id(), LABEL2.id()]);
                continue;
            }

            if ui
                .add(Checkbox::new(CHECKBOX_1, "Checkbox1", &mut self.checkbox1))
                .is_value_changed()
            {
                log::info!("Checkbox1 changed: {}", self.checkbox1);
                let tip: &str = &TipOnOff(self.checkbox1);
                self.label1.clear();
                write!(&mut self.label1, "{}", tip).unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                continue;
            }
            if ui
                .add(Checkbox::new(CHECKBOX_2, "Checkbox2", &mut self.checkbox2))
                .is_value_changed()
            {
                log::info!("Checkbox2 changed: {}", self.checkbox2);
                Languages::switch_language();

                let lan: &str = Languages::get_language().into();
                self.label1.clear();
                write!(&mut self.label1, "{}", lan).unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                continue;
            }

            if ui
                .add(RadioButton::new(
                    RADIOBUTTON1,
                    "Radio1",
                    RadioGroup::Btn1,
                    &mut self.radio,
                ))
                .is_value_changed()
            {
                log::info!("Radio1 clicked {:?}", self.radio);
                self.label1.clear();
                write!(&mut self.label1, "Radio1").unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                self.widget_states
                    .force_redraw_range(RADIOBUTTON1.id(), RADIOBUTTON3.id());
                continue;
            }

            if ui
                .add(RadioButton::new(
                    RADIOBUTTON2,
                    "Radio2",
                    RadioGroup::Btn2,
                    &mut self.radio,
                ))
                .is_value_changed()
            {
                log::info!("Radio2 clicked {:?}", self.radio);
                self.label1.clear();
                write!(&mut self.label1, "Radio2").unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                self.widget_states.force_redraw_multi(RADIOBUTTON_IDS);
                continue;
            }
            if ui
                .add(RadioButton::new(
                    RADIOBUTTON3,
                    "Radio3",
                    RadioGroup::Btn3,
                    &mut self.radio,
                ))
                .is_value_changed()
            {
                log::info!("Radio3 clicked {:?}", self.radio);
                self.label1.clear();
                write!(&mut self.label1, "Radio3").unwrap();
                self.widget_states.force_redraw(LABEL1.id());
                self.widget_states.force_redraw_multi(RADIOBUTTON_IDS);
                continue;
            }

            break;
        }
    }
}
