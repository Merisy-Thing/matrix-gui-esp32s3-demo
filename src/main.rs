#![no_std]
#![no_main]

mod cst816d;
mod pages;

use crate::cst816d::{CST816D, TouchPoint, TouchState};
use crate::pages::{basic_example::BasicExample, msg_box::MsgBox};
use core::cell::RefCell;
use critical_section::Mutex;
use dummy_pin::DummyPin;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use embedded_graphics::image::ImageRaw;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Event, Input, InputConfig, Level, Output, OutputConfig},
    i2c::master::{BusTimeout, Config as I2cConfig, I2c},
    interrupt::software::SoftwareInterruptControl,
    ram,
    spi::{
        Mode,
        master::{Config as SpiConfig, Spi},
    },
    time::Rate,
    timer::timg::TimerGroup,
};

use matrix_gui::prelude::*;
use multi_mono_font::{CharSize, MultiMonoFont, mapping::StrGlyphMapping};
use st7789_lcd::*;

esp_bootloader_esp_idf::esp_app_desc!();

pub struct LcdCfg;

impl LcdConfig for LcdCfg {
    const WIDTH: u16 = 280;
    const HEIGHT: u16 = 240;
    const X_OFFSET: u16 = 20;
    const Y_OFFSET: u16 = 0;
    const LITTLE_ENDIAN: bool = false;
}

const HOR_RES: u16 = LcdCfg::WIDTH;
const VER_RES: u16 = LcdCfg::HEIGHT;

static TP_DEV: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

static TOUCH_EVENT: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static TOUCH_DATA: Signal<CriticalSectionRawMutex, TouchState> = Signal::new();

#[esp_hal::handler]
#[ram]
fn handler() {
    critical_section::with(|cs| {
        let mut binding = TP_DEV.borrow_ref_mut(cs);
        if let Some(tp_dev) = binding.as_mut()
            && tp_dev.is_interrupt_set()
        {
            //log::info!("touch interrupt");
            TOUCH_EVENT.signal(());
            tp_dev.clear_interrupt();
        }
    });
}

#[embassy_executor::task]
async fn task_touch(mut touch: CST816D<I2c<'static, esp_hal::Blocking>>) {
    log::info!("touch task start");
    loop {
        TOUCH_EVENT.wait().await;
        if let Ok(state) = touch.read_touch() {
            //log::info!("touch irq state = {:?}", state);
            TOUCH_DATA.signal(state);
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Pages {
    Basic,
    MsgBox,
}

type PageSw = Signal<NoopRawMutex, Pages>;

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    #[cfg(feature = "log")]
    {
        esp_println::logger::init_logger(log::LevelFilter::Info);
    }

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    let sw_int = SoftwareInterruptControl::new(p.SW_INTERRUPT);
    let timg0 = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    log::info!("hello world!");

    let mut io = esp_hal::gpio::Io::new(p.IO_MUX);
    io.set_interrupt_handler(handler);

    let mut led = Output::new(p.GPIO10, Level::High, OutputConfig::default());
    let mut tp_rst = Output::new(p.GPIO18, Level::High, OutputConfig::default());
    let mut tp_int = Input::new(p.GPIO21, InputConfig::default());

    let tp_scl = p.GPIO16;
    let tp_sda = p.GPIO17;

    let touch = {
        let i2c = I2c::new(
            p.I2C0,
            I2cConfig::default()
                .with_frequency(Rate::from_khz(400))
                .with_timeout(BusTimeout::BusCycles(200)),
        )
        .unwrap()
        .with_sda(tp_sda)
        .with_scl(tp_scl);

        let mut touch = CST816D::new(i2c);

        let mut delay = Delay::new();
        tp_rst.set_low();
        delay.delay_ms(10);
        tp_rst.set_high();
        delay.delay_ms(300);

        if let Err(ret) = touch.init() {
            log::error!("# touch.init: {:?}", ret);
        } else {
            log::info!("# touch.init success");
        }

        touch.set_size(HOR_RES, VER_RES);
        touch.rotate(false);

        touch
    };
    critical_section::with(|cs| {
        tp_int.listen(Event::FallingEdge);
        TP_DEV.borrow_ref_mut(cs).replace(tp_int)
    });

    match task_touch(touch) {
        Ok(tt) => spawner.spawn(tt),
        Err(ret) => log::error!("touch task error: {:?}", ret),
    }

    let mut lcd_bl = Output::new(p.GPIO11, Level::High, OutputConfig::default());
    lcd_bl.set_high();

    let mut display = {
        let rst = Output::new(p.GPIO15, Level::High, OutputConfig::default());
        let dc = Output::new(p.GPIO12, Level::High, OutputConfig::default());
        let spi_bus = Spi::new(
            p.SPI2,
            SpiConfig::default()
                .with_frequency(Rate::from_mhz(20))
                .with_mode(Mode::_3),
        )
        .unwrap()
        .with_sck(p.GPIO13)
        .with_mosi(p.GPIO14);

        let cs = DummyPin::new_high();
        let lcd_spi = ExclusiveDevice::new(spi_bus, cs, NoDelay).unwrap();
        let mut st7789 = ST7789::new(lcd_spi, dc, rst, LcdCfg);
        let mut disp_delay = Delay::new();
        st7789
            .init(&mut disp_delay, Orientation::PortraitSwapped, true, true)
            .unwrap();
        st7789
    };

    display.on().unwrap();

    TOUCH_DATA.signal(TouchState::Released(TouchPoint::new(0, 0)));
    let pages_sw: PageSw = Signal::new();

    log::info!("loop!");
    let mut basic = BasicExample::new(&pages_sw);
    let mut msg_box = MsgBox::new(&pages_sw);
    let mut curr_page = Pages::Basic;

    loop {
        if let Some(tp) = TOUCH_DATA.try_take() {
            let (tp_down, location) = match tp {
                TouchState::Pressed(touch_point) => (true, touch_point.into()),
                TouchState::Released(touch_point) => (false, touch_point.into()),
            };
            let tick = embassy_time::Instant::now();

            match curr_page {
                Pages::Basic => {
                    basic.update(tp_down, location, &mut display);
                }
                Pages::MsgBox => {
                    msg_box.update(tp_down, location, &mut display);
                }
            }

            log::info!("update cost: {}ms", tick.elapsed().as_millis());
        }

        if let Some(page) = pages_sw.try_take() {
            curr_page = page;
            match page {
                Pages::Basic => {
                    basic.redraw();
                    basic.update(false, Point::zero(), &mut display);
                }
                Pages::MsgBox => {
                    msg_box.redraw();
                    msg_box.update(false, Point::zero(), &mut display);
                }
            }
        }

        led.toggle();
        Timer::after(Duration::from_millis(10)).await;
    }
}

impl From<TouchPoint> for Point {
    fn from(tp: TouchPoint) -> Self {
        Self::new(tp.x as i32, tp.y as i32)
    }
}

const GB2313_TIER1_16X16_FONT: MultiMonoFont = MultiMonoFont {
    image: ImageRaw::new(include_bytes!("../assets/GB2313_Tier1_16x16_11.bin"), 3600),
    glyph_mapping: &StrGlyphMapping::new(include_str!("../assets/GB2313_Tier1.txt"), 0),
    character_size: CharSize::new(16, 16),
    character_spacing: 0,
    baseline: 16,
};
const TXT_FONT: UiFont = &[&multi_mono_font::ascii::FONT_9X18, &GB2313_TIER1_16X16_FONT];

pub const fn example_style() -> &'static Style<Rgb565> {
    // const STYLE: Style<Rgb565> = Style {
    //     background_color: Rgb565::new(0x4, 0x8, 0x4),
    //     border_color: Rgb565::RED,
    //     text_color: Rgb565::WHITE,
    //     border_width: 1,
    //     default_font: TXT_FONT,
    //     default_padding: Size::new(2, 2),
    //     corner_radius: 5,
    // };

    const STYLE: Style<Rgb565> = Style {
        background_color: Rgb565::BLACK,
        border_color: Rgb565::RED,
        text_color: Rgb565::WHITE,
        border_width: 1,
        default_font: TXT_FONT,
        default_padding: Size::new(4, 4),
        corner_radius: 5,
    };

    &STYLE
}
