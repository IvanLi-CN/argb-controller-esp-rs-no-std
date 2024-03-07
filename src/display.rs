// use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
// use embassy_time::Delay;
// use embedded_graphics::pixelcolor::Rgb565;
// use embedded_graphics::prelude::*;

// use embassy_sync::blocking_mutex::raw::RawMutex;
// use embassy_sync::mutex::Mutex;
// use embedded_hal_bus::spi::NoDelay;
// use esp_println::println;
// use hal::clock;
// use hal::dma::ChannelTypes;
// use hal::dma::SpiPeripheral;
// use hal::gpio::GpioPin;
// use hal::gpio::Output;
// use hal::gpio::PushPull;
// use hal::gpio::Unknown;
// use hal::peripherals::SPI2;
// use hal::spi::master::dma::SpiDma;
// use hal::spi::FullDuplexMode;
// use st7735_embassy::ST7735;

// pub async fn draw_ferris<D>(
//     spi: &mut SpiDevice<'_, RawMutex, SPI2, Option<Unknown>>,
//     dc: GpioPin<Output<PushPull>, 3>,
//     rst: GpioPin<Output<PushPull>, 4>,
// ) {
//     let rgb = true;
//     let inverted = false;
//     let width = 80;
//     let height = 160;

//     let mut display = ST7735::new(
//         spi,
//         dc,
//         rst,
//         st7735_embassy::Config::Default::default(),
//         width,
//         height,
//     );

//     println!("lcd init...");

//     display.init(&mut Delay).await.unwrap();
//     display.clear(Rgb565::BLACK).unwrap();
//     // display.set_offset(0, 25);

//     // let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("./assets/ferris.raw"), 86);
//     // let image = Image::new(&image_raw, Point::new(26, 8));
//     // image.draw(&mut display).unwrap();

//     println!("lcd test have done.");
//     loop {
//         continue;
//     }
// }
