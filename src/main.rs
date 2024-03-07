#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::Drawable;
use esp_backtrace as _;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::WifiStaDevice;
use esp_wifi::{initialize, EspWifiInitFor};
use hal::dma::DmaPriority;
use hal::gdma::Gdma;
use hal::{
    clock::{self, ClockControl},
    embassy,
    gpio::{GpioPin, Unknown},
    peripheral::Peripheral,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    timer::TimerGroup,
    Rng, IO,
};
use st7735::ST7735;
use static_cell::make_static;

use embassy_net::{Config, Stack, StackResources};
mod openwrt;
mod openwrt_types;
mod wifi;
use openwrt::netdata_info;
use wifi::{connection, get_ip_addr, net_task};

use esp_backtrace as _;
#[embassy_executor::task]
async fn blink(blink_led: &'static mut GpioPin<Unknown, 1>) {
    let mut blink_led = unsafe { blink_led.clone_unchecked() }.into_push_pull_output();
    blink_led.set_high().unwrap();

    loop {
        blink_led.toggle().unwrap();
        Timer::after(Duration::from_millis(5_00)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks: clock::Clocks<'static> = ClockControl::max(system.clock_control).freeze();
    // let clocks: &'static _ = make_static!(clocks);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let blink_led: &'static mut GpioPin<Unknown, 1> = make_static!(io.pins.gpio1);

    embassy::init(&clocks, timer_group0);

    let timer = hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    // Wi-Fi

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();
    let config = Config::dhcpv4(Default::default());
    let seed = 1234; // very random, very secure seed

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    // DMA

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    // SPI

    let sck = io.pins.gpio8.into_push_pull_output();
    let sdo = io.pins.gpio10.into_push_pull_output();
    let descriptors = make_static!([0u32; 8 * 3]);
    let rx_descriptors = make_static!([0u32; 8 * 3]);
    let spi = Spi::new(peripherals.SPI2, 40u32.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sck)
        .with_mosi(sdo)
        .with_dma(dma_channel.configure(
            false,
            descriptors,
            rx_descriptors,
            DmaPriority::Priority0,
        ));
    let rst = io.pins.gpio3.into_push_pull_output();
    let dc = io.pins.gpio4.into_push_pull_output();
    let lcd_cs = io.pins.gpio5.into_push_pull_output();
    // let spi = make_static!(spi);
    let spi: Mutex<NoopRawMutex, _> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi, lcd_cs);
    // let rst = make_static!(rst);
    // let dc = make_static!(dc);

    spawner.spawn(blink(blink_led)).ok();
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(&stack)).ok();
    spawner.spawn(get_ip_addr(&stack)).ok();
    spawner.spawn(netdata_info(&stack)).ok();

    let rgb = false;
    let inverted = false;
    let width = 160;
    let height = 80;

    println!("lcd init...");

    let mut display = ST7735::new(
        spi_dev,
        dc,
        rst,
        st7735::Config {
            rgb,
            inverted,
            orientation: st7735::Orientation::Landscape,
        },
        width,
        height,
    );
    display.init(&mut Delay).await.unwrap();
    display.set_offset(0, 24);
    display.clear(Rgb565::BLUE).unwrap();

    let image_raw: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("./assets/ferris.raw"), 86);
    let image = Image::new(&image_raw, Point::new(32, 8));

    println!("lcd test have done.");

    let mut color_r = 0u8;
    let mut color_g = 0u8;
    let mut color_b = 0u8;

    loop {
        color_r = color_r.wrapping_add(29);
        color_g = color_g.wrapping_add(33);
        color_b = color_b.wrapping_add(37);
        let color = Rgb565::new(color_r, color_g, color_b);
        Timer::after(Duration::from_millis(1_000)).await;
        display.flush().await.unwrap();
        display.clear(color).unwrap();
        Timer::after(Duration::from_millis(1_000)).await;
        image.draw(&mut display).unwrap();
        display.flush().await.unwrap();
    }
}
