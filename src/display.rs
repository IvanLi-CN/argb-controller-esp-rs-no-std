use crate::bus::{
    NetSpeed, WiFiConnectStatus, NET_SPEED, WIFI_CONNECT_STATUS,
};
use core::future::Future;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Instant, Timer};
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle};
use embedded_graphics::text::TextStyle;
use embedded_graphics::{
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use hal::dma::Channel0;
use hal::gpio::GpioPin;
use hal::peripherals::SPI2;
use hal::spi::master::dma::SpiDma;
use hal::spi::FullDuplexMode;
use heapless::String;
use numtoa::NumToA;
use st7735::ST7735;
use static_cell::make_static;

pub(crate) type DisplayST7735 = ST7735<
    SpiDevice<
        'static,
        NoopRawMutex,
        SpiDma<'static, SPI2, Channel0, FullDuplexMode>,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 9>,
    >,
    GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 7>,
    GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 8>,
>;

const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("./assets/seven-segment-font.raw"), 224),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(22, 40),
    character_spacing: 4,
    baseline: 7,
    underline: DecorationDimensions::default_underline(40),
    strikethrough: DecorationDimensions::default_strikethrough(40),
};

#[embassy_executor::task]
pub(crate) async fn init_display(display: &'static mut DisplayST7735) {
    let mut gui: GUI<'static> = GUI::new(display);

    gui.init().await;

    let gui = make_static!(Mutex::<NoopRawMutex, GUI<'static>>::new(gui));

    let mut wifi_status: WiFiConnectStatus;

    loop {
        let wifi_status_guard = WIFI_CONNECT_STATUS.lock().await;
        wifi_status = *wifi_status_guard;
        drop(wifi_status_guard);

        let mut gui = gui.lock().await;
        match wifi_status {
            WiFiConnectStatus::Connecting => gui.wifi_connecting_display().await,
            WiFiConnectStatus::Connected => gui.network_speed().await,
            _ => {}
        };
        drop(gui);

        Timer::after_millis(10).await;
    }
}

enum DisplayPage<'a> {
    Init,
    Connecting(WiFiConnectingPage<'a>),
    NetworkSpeed(NetDataTrafficSpeedPage<'a>),
}

struct GUI<'a> {
    display: &'a mut DisplayST7735,
    page: DisplayPage<'a>,
}

impl<'a> GUI<'a> {
    fn new(display: &'a mut DisplayST7735) -> Self {
        Self {
            display,
            page: DisplayPage::Init,
        }
    }

    pub async fn init(&mut self) {
        self.display.init(&mut Delay).await.unwrap();
        self.display.set_offset(0, 24);
        self.display.clear(Rgb565::BLACK).unwrap();
        self.display.flush().await.unwrap();

        let image_raw: ImageRawLE<Rgb565> =
            ImageRaw::new(include_bytes!("./assets/simple-icons_espressif.raw"), 24);
        let image = Image::new(&image_raw, Point::new(160 - 60, 80 - 24 - 8));
        image.draw(self.display).unwrap();

        let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(
            include_bytes!("./assets/vscode-icons_file-type-rust.raw"),
            24,
        );
        let image = Image::new(&image_raw, Point::new(160 - 24 - 8, 80 - 24 - 8));
        image.draw(self.display).unwrap();

        self.display.flush().await.unwrap();
    }

    pub async fn wifi_connecting_display(&mut self) {
        if !matches!(self.page, DisplayPage::Connecting(_)) {
            self.page = DisplayPage::Connecting(WiFiConnectingPage::new());
        }

        if let DisplayPage::Connecting(ref mut page) = self.page {
            page.frame(&mut self.display).await;
        }
    }

    pub async fn network_speed(&mut self) {
        if !matches!(self.page, DisplayPage::NetworkSpeed(_)) {
            self.page = DisplayPage::NetworkSpeed(NetDataTrafficSpeedPage::new());
        }

        if let DisplayPage::NetworkSpeed(ref mut page) = self.page {
            page.frame(&mut self.display).await;
        }
    }
}

trait GUIPageFrame {
    fn frame(&mut self, display: &mut DisplayST7735) -> impl Future<Output = ()>;
}

struct WiFiConnectingPage<'a> {
    animation_frame_index: u8,
    last_draw_time: Instant,
    character_style: MonoTextStyle<'a, Rgb565>,
    text_style: TextStyle,
    string: String<20>,
}

impl<'a> WiFiConnectingPage<'a> {
    pub fn new() -> Self {
        Self {
            animation_frame_index: 0,
            last_draw_time: Instant::MIN,
            character_style: MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
            text_style: TextStyleBuilder::new()
                .baseline(Baseline::Middle)
                .alignment(Alignment::Left)
                .build(),
            string: String::new(),
        }
    }
}

impl<'a> GUIPageFrame for WiFiConnectingPage<'a> {
    async fn frame(&mut self, display: &mut DisplayST7735) {
        if self.last_draw_time.elapsed().as_millis() < 200 {
            return;
        }

        display.clear(Rgb565::BLACK).unwrap();

        self.last_draw_time = Instant::now();

        self.string.clear();
        self.string.push_str("Connecting").unwrap();

        for _ in 0..self.animation_frame_index {
            self.string.push_str(".").unwrap();
        }

        let image_raw: ImageRawLE<Rgb565> = match self.animation_frame_index {
            0 => {
                self.animation_frame_index = 1;
                ImageRaw::new(include_bytes!("./assets/ci_wifi-none.raw"), 32)
            }
            1 => {
                self.animation_frame_index = 2;
                ImageRaw::new(include_bytes!("./assets/ci_wifi-low.raw"), 32)
            }
            2 => {
                self.animation_frame_index = 3;
                ImageRaw::new(include_bytes!("./assets/ci_wifi-medium.raw"), 32)
            }
            _ => {
                self.animation_frame_index = 0;
                ImageRaw::new(include_bytes!("./assets/ci_wifi-high.raw"), 32)
            }
        };
        let image = Image::new(&image_raw, Point::new(0, 24));
        image.draw(display).unwrap();

        Text::with_text_style(
            self.string.as_str(),
            Point::new(30, 40),
            self.character_style,
            self.text_style,
        )
        .draw(display)
        .unwrap();

        display.flush().await.unwrap();
    }
}

struct NetDataTrafficSpeedPage<'a> {
    character_style: MonoTextStyle<'a, Rgb565>,
    text_style: TextStyle,
    prev_speed: NetSpeed,
    str_buff: [u8; 20],
    string: String<20>,
}

impl<'a> NetDataTrafficSpeedPage<'a> {
    pub fn new() -> Self {
        Self {
            character_style: MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
            text_style: TextStyleBuilder::new()
                .baseline(Baseline::Bottom)
                .alignment(Alignment::Right)
                .build(),
            prev_speed: NetSpeed::default(),
            str_buff: [0u8; 20],
            string: String::new(),
        }
    }
}

impl<'a> GUIPageFrame for NetDataTrafficSpeedPage<'a> {
    async fn frame(&mut self, display: &mut DisplayST7735) {
        let curr_speed_guard = NET_SPEED.lock().await;
        let curr_speed = *curr_speed_guard;

        self.prev_speed = curr_speed;
        drop(curr_speed_guard);

        // Direct

        {
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Rgb565::CSS_ORANGE_RED)
                .fill_color(Rgb565::CSS_DARK_RED)
                .build();
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(0, 0), Size::new(79, 24)),
                Size::new(5, 5),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Rgb565::CSS_BLUE)
                .fill_color(Rgb565::CSS_DARK_BLUE)
                .build();
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(81, 0), Size::new(79, 24)),
                Size::new(5, 5),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();

            // UP

            curr_speed.get_direct_up_bps_str(&mut self.string, &mut self.str_buff);

            Text::with_text_style(
                self.string.as_str(),
                Point::new(75, 22),
                self.character_style,
                self.text_style,
            )
            .draw(display)
            .unwrap();

            // DOWN

            
            curr_speed.get_direct_down_bps_str(&mut self.string, &mut self.str_buff);

            Text::with_text_style(
                self.string.as_str(),
                Point::new(155, 22),
                self.character_style,
                self.text_style,
            )
            .draw(display)
            .unwrap();
        }

        // Proxy
        {
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Rgb565::CSS_YELLOW)
                .fill_color(Rgb565::CSS_GOLD)
                .build();
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(0, 29), Size::new(79, 24)),
                Size::new(5, 5),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Rgb565::CSS_LIME_GREEN)
                .fill_color(Rgb565::CSS_FOREST_GREEN)
                .build();
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(81, 29), Size::new(79, 24)),
                Size::new(5, 5),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();

            // UP

            curr_speed.get_proxy_up_bps_str(&mut self.string, &mut self.str_buff);

            Text::with_text_style(
                self.string.as_str(),
                Point::new(75, 51),
                self.character_style,
                self.text_style,
            )
            .draw(display)
            .unwrap();

            // DOWN

            curr_speed.get_proxy_down_bps_str(&mut self.string, &mut self.str_buff);

            Text::with_text_style(
                self.string.as_str(),
                Point::new(155, 51),
                self.character_style,
                self.text_style,
            )
            .draw(display)
            .unwrap();
        }

        // Bypass
        {
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Rgb565::CSS_SLATE_BLUE)
                .fill_color(Rgb565::CSS_PURPLE)
                .build();
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(0, 56), Size::new(79, 24)),
                Size::new(5, 5),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();
            let style = PrimitiveStyleBuilder::new()
                .stroke_width(1)
                .stroke_color(Rgb565::CSS_LIME_GREEN)
                .fill_color(Rgb565::CSS_FOREST_GREEN)
                .build();
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(81, 56), Size::new(79, 24)),
                Size::new(5, 5),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();

            // UP

            curr_speed.get_bypass_up_bps_str(&mut self.string, &mut self.str_buff);

            Text::with_text_style(
                self.string.as_str(),
                Point::new(75, 78),
                self.character_style,
                self.text_style,
            )
            .draw(display)
            .unwrap();

            // DOWN

            curr_speed.get_bypass_down_bps_str(&mut self.string, &mut self.str_buff);

            Text::with_text_style(
                self.string.as_str(),
                Point::new(155, 78),
                self.character_style,
                self.text_style,
            )
            .draw(display)
            .unwrap();
        }

        display.flush().await.unwrap();
    }
}
