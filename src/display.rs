use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use esp_println::println;
use hal::gdma::Channel0;
use hal::gpio::GpioPin;
use hal::peripherals::SPI2;
use hal::spi::master::dma::SpiDma;
use hal::spi::FullDuplexMode;
use st7735::ST7735;

#[embassy_executor::task]
pub(crate) async fn init_display(
    display: &'static mut ST7735<
        SpiDevice<
            '_,
            NoopRawMutex,
            SpiDma<'_, SPI2, Channel0, FullDuplexMode>,
            GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 5>,
        >,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 4>,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 3>,
    >,
) {
    display.init(&mut Delay).await.unwrap();
    display.set_offset(0, 24);
    display.clear(Rgb565::BLUE).unwrap();
    display.flush().await.unwrap();

    let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("./assets/ferris.raw"), 86);
    let image = Image::new(&image_raw, Point::new(32, 8));

    println!("lcd test have done");

    let mut color_r = 0u8;
    let mut color_g = 0u8;
    let mut color_b = 0u8;

    loop {
        color_r = color_r.wrapping_add(29);
        color_g = color_g.wrapping_add(33);
        color_b = color_b.wrapping_add(37);
        let color = Rgb565::new(color_r, color_g, color_b);
        display.clear(color).unwrap();
        image.draw(display).unwrap();
        display.flush().await.unwrap();
    }
}
