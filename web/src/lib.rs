use std::sync::atomic::{AtomicBool, Ordering};

use bare_sync::{signal::Signal, NoopSyncRawMutex};
use embedded_graphics::{geometry::Point, pixelcolor::Rgb565};
use embedded_graphics_web_simulator::{
    display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
};
use matrix_gui_demo_pages::{self as demo, Pages};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::{console, window, Element, EventTarget, MouseEvent};

static MOUSE_CLICK_EVENT: Signal<NoopSyncRawMutex, ClickEvent> = Signal::new();
static MOUSE_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone, Debug)]
pub enum ClickEvent {
    Pressed(Point),
    Released(Point),
}

const CANVAS_WIDTH: i32 = 280;
const CANVAS_HEIGHT: i32 = 240;

pub fn add_mouse_click(canvas_x: i32, canvas_y: i32, canvas: &Element) {
    if let Some(event_target) = canvas.dyn_ref::<EventTarget>() {
        let down_closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let x = event.client_x() - canvas_x;
            let y = event.client_y() - canvas_y;
            if x >= 0 && y >= 0 && x < CANVAS_WIDTH && y < CANVAS_HEIGHT {
                MOUSE_DOWN.store(true, Ordering::Relaxed);
                MOUSE_CLICK_EVENT.signal(ClickEvent::Pressed(Point::new(x, y)));
            }
        }) as Box<dyn FnMut(_)>);
        event_target
            .add_event_listener_with_callback("mousedown", down_closure.as_ref().unchecked_ref())
            .unwrap();
        down_closure.forget();

        let up_closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let x = event.client_x() - canvas_x;
            let y = event.client_y() - canvas_y;
            if x >= 0 && y >= 0 && x < CANVAS_WIDTH && y < CANVAS_HEIGHT {
                MOUSE_DOWN.store(false, Ordering::Relaxed);
                MOUSE_CLICK_EVENT.signal(ClickEvent::Released(Point::new(x, y)));
            }
        }) as Box<dyn FnMut(_)>);
        event_target
            .add_event_listener_with_callback("mouseup", up_closure.as_ref().unchecked_ref())
            .unwrap();
        up_closure.forget();
    }
}

pub fn add_mouse_move(canvas_x: i32, canvas_y: i32, canvas: &Element) {
    if let Some(event_target) = canvas.dyn_ref::<EventTarget>() {
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            if MOUSE_DOWN.load(Ordering::Relaxed) {
                let x = event.client_x() - canvas_x;
                let y = event.client_y() - canvas_y;
                if x >= 0 && y >= 0 && x < CANVAS_WIDTH && y < CANVAS_HEIGHT {
                    MOUSE_CLICK_EVENT.signal(ClickEvent::Pressed(Point::new(x, y)));
                }
            }
        }) as Box<dyn FnMut(_)>);
        event_target
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let output_settings = OutputSettingsBuilder::new().build();
    let display = WebSimulatorDisplay::new(
        (CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32),
        &output_settings,
        None,
    );

    let document = window().unwrap().document().unwrap();
    let container = document.get_elements_by_tag_name("CANVAS");

    if let Some(canvas) = container.item(0) {
        let rect = canvas.get_bounding_client_rect();
        let canvas_x = rect.x() as i32;
        let canvas_y = rect.y() as i32;
        console::log_1(&format!("Canvas position: x={}, y={}", canvas_x, canvas_y).into());

        add_mouse_move(canvas_x, canvas_y, &canvas);
        add_mouse_click(canvas_x, canvas_y, &canvas);
    }

    start_demo_task(display).unwrap();

    Ok(())
}

static PAGES_SW: demo::PageSw = bare_sync::signal::Signal::new();
const TICK_INTERVAL: u64 = 20;

pub fn start_demo_task(mut display: WebSimulatorDisplay<Rgb565>) -> Result<(), JsValue> {
    let mut home = demo::HomePage::new(&PAGES_SW);
    let mut basic = demo::BasicExample::new(&PAGES_SW);
    let mut msg_box = demo::MsgBox::new(&PAGES_SW);
    let mut calculator = demo::Calculator::new(&PAGES_SW);
    let mut anim_switch = demo::AnimSwitch::new(&PAGES_SW);
    let mut curr_page = Pages::Home;
    let tick_count = std::rc::Rc::new(std::cell::RefCell::new(0_u64));
    let tick_count_clone = tick_count.clone();
    let last_inst = std::rc::Rc::new(std::cell::RefCell::new(0_u64));
    let last_inst_clone = last_inst.clone();

    MOUSE_CLICK_EVENT.signal(ClickEvent::Pressed(Point::new(0, 0)));

    let closure = Closure::wrap(Box::new(move || {
        let mut now = tick_count_clone.borrow_mut();
        *now += TICK_INTERVAL;

        if curr_page == Pages::AnimSwitch {
            let mut last_inst = last_inst_clone.borrow_mut();
            let delta = *now - *last_inst;
            if delta > 30 {
                *last_inst = *now;
                anim_switch
                    .update_animations(core::time::Duration::from_millis(delta), &mut display);

                display.flush().expect("could not flush buffer");
            }
        }

        if let Some(event) = MOUSE_CLICK_EVENT.try_take() {
            let (tp_down, location) = match event {
                ClickEvent::Pressed(pos) => (true, pos.into()),
                ClickEvent::Released(pos) => (false, pos.into()),
            };

            match curr_page {
                Pages::Home => {
                    home.update(tp_down, location, &mut display);
                }
                Pages::Basic => {
                    basic.update(tp_down, location, &mut display);
                }
                Pages::MsgBox => {
                    msg_box.update(tp_down, location, &mut display);
                }
                Pages::Calculator => {
                    calculator.update(tp_down, location, &mut display);
                }
                Pages::AnimSwitch => {
                    anim_switch.update(tp_down, location, &mut display);
                }
            }

            display.flush().expect("could not flush buffer");
        }

        if let Some(page) = PAGES_SW.try_take() {
            curr_page = page;
            match page {
                Pages::Home => {
                    home.redraw();
                    home.update(false, Point::zero(), &mut display);
                }
                Pages::Basic => {
                    basic.redraw();
                    basic.update(false, Point::zero(), &mut display);
                }
                Pages::MsgBox => {
                    msg_box.redraw();
                    msg_box.update(false, Point::zero(), &mut display);
                }
                Pages::Calculator => {
                    calculator.redraw();
                    calculator.update(false, Point::zero(), &mut display);
                }
                Pages::AnimSwitch => {
                    anim_switch.redraw();
                    anim_switch.update(false, Point::zero(), &mut display);
                }
            }

            display.flush().expect("could not flush buffer");
        }
    }) as Box<dyn FnMut()>);

    let window = window().expect("no global `window` exists");

    window.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        TICK_INTERVAL as i32,
    )?;

    closure.forget();

    Ok(())
}
