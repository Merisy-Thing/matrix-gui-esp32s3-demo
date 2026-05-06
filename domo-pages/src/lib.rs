#![no_std]

pub mod pages;

use bare_sync::{NoopSyncRawMutex, signal::Signal};
use embedded_graphics::image::ImageRaw;
use matrix_gui::prelude::*;
use multi_mono_font::{CharSize, MultiMonoFont, mapping::StrGlyphMapping};

pub use pages::*;

pub type PageSw = Signal<NoopSyncRawMutex, Pages>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pages {
    Home,
    Basic,
    MsgBox,
    Calculator,
    AnimSwitch,
}

impl core::fmt::Display for Pages {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Pages::Home => write!(f, "Home"),
            Pages::Basic => write!(f, "Basic"),
            Pages::MsgBox => write!(f, "MsgBox"),
            Pages::Calculator => write!(f, "Calculator"),
            Pages::AnimSwitch => write!(f, "AnimSwitch"),
        }
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
