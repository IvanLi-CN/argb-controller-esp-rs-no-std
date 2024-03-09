use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pubsub::WaitResult;
use embassy_time::Delay;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::{
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use esp_println::println;
use hal::gdma::Channel0;
use hal::gpio::GpioPin;
use hal::peripherals::SPI2;
use hal::spi::master::dma::SpiDma;
use hal::spi::FullDuplexMode;
use heapless::String;
use numtoa::NumToA;
use st7735::ST7735;

use crate::bus::NET_DATA_TRAFFIC_SPEED_PUB_SUB;

const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("./assets/seven-segment-font.raw"), 224),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(22, 40),
    character_spacing: 4,
    baseline: 7,
    underline: DecorationDimensions::default_underline(40),
    strikethrough: DecorationDimensions::default_strikethrough(40),
};

pub(crate) type DisplayST7735 = ST7735<
    SpiDevice<
        'static,
        NoopRawMutex,
        SpiDma<'static, SPI2, Channel0, FullDuplexMode>,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 5>,
    >,
    GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 4>,
    GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 3>,
>;

#[embassy_executor::task]
pub(crate) async fn init_display(display: &'static mut DisplayST7735) {
    display.init(&mut Delay).await.unwrap();
    display.set_offset(0, 24);
    display.clear(Rgb565::BLACK).unwrap();
    display.flush().await.unwrap();

    let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("./assets/rust_logo.bin"), 64);
    let image = Image::new(&image_raw, Point::new((160 - 64) / 2, (80 - 64) / 2));

    image.draw(display).unwrap();
    display.flush().await.unwrap();

    println!("lcd test have done");

    let spawner = Spawner::for_current_executor().await;
    spawner.spawn(network_speed(display)).unwrap();
}

#[embassy_executor::task]
pub(crate) async fn network_speed(display: &'static mut DisplayST7735) {
    let mut character_style = MonoTextStyle::new(&SEVENT_SEGMENT_FONT, Rgb565::CYAN);
    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Right)
        .build();

    let mut subscriber: embassy_sync::pubsub::Subscriber<
        '_,
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        crate::bus::NetDataTrafficSpeed,
        4,
        4,
        1,
    > = NET_DATA_TRAFFIC_SPEED_PUB_SUB.subscriber().unwrap();
    let mut str_buff = [0u8; 20];
    let mut string: String<20> = String::new();

    println!("start network speed task");

    loop {
        let msg = subscriber.next_message().await;

        match msg {
            WaitResult::Lagged(lag) => {
                println!("lagged {}", lag);
            }
            WaitResult::Message(speed) => {
                display.clear(Rgb565::BLACK).unwrap();

                // UP

                character_style.set_text_color(Some(Rgb565::CSS_ORANGE_RED));

                string.clear();
                string
                    .push_str(speed.up.numtoa_str(10, &mut str_buff))
                    .unwrap();

                println!("speed: {}", string);
                Text::with_text_style(
                    string.as_str(),
                    display.bounding_box().center(),
                    character_style,
                    text_style,
                )
                .translate(Point::new(80, 0))
                .draw(display)
                .unwrap();

                // DOWN

                character_style.set_text_color(Some(Rgb565::CYAN));

                string.clear();
                string
                    .push_str(speed.down.numtoa_str(10, &mut str_buff))
                    .unwrap();

                println!("speed: {}", string);
                Text::with_text_style(
                    string.as_str(),
                    display.bounding_box().center(),
                    character_style,
                    text_style,
                )
                .translate(Point::new(80, 40))
                .draw(display)
                .unwrap();

                display.flush().await.unwrap();
            }
        }
    }
}
