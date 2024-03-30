#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(const_refs_to_static)]
#![feature(const_maybe_uninit_write)]
#![feature(const_mut_refs)]
#![feature(int_roundings)]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::WifiStaDevice;
use esp_wifi::{initialize, EspWifiInitFor};
use hal::dma::{Channel0, Dma};
use hal::dma::DmaPriority;
use hal::dma_descriptors;
use hal::peripherals::SPI2;
use hal::spi::master::dma::SpiDma;
use hal::spi::FullDuplexMode;
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
mod bus;
mod display;
mod wifi;
mod udp_client;
use wifi::{connection, get_ip_addr, net_task};

use esp_backtrace as _;

use crate::udp_client::receiving_net_speed;
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
    esp_println::println!("Init main");

    // Basic stuff

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks: clock::Clocks<'static> = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let blink_led: &'static mut GpioPin<Unknown, 1> = make_static!(io.pins.gpio1);

    embassy::init(&clocks, timer_group0);

    let timer = hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;

    // Wi-Fi

    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
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

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    // SPI

    let sdo = io.pins.gpio5.into_push_pull_output();
    let sck = io.pins.gpio6.into_push_pull_output();
    let (tx_descriptors,  rx_descriptors) = dma_descriptors!(32000, 4096);
    let tx_descriptors = make_static!(tx_descriptors);
    let rx_descriptors = make_static!(rx_descriptors);

    let spi: SpiDma<'_, SPI2, Channel0, FullDuplexMode> =
        Spi::new(peripherals.SPI2, 40u32.MHz(), SpiMode::Mode0, &clocks)
            .with_sck(sck)
            .with_mosi(sdo)
            .with_dma(dma_channel.configure(
                false,
                tx_descriptors,
                rx_descriptors,
                DmaPriority::Priority0,
            ));
    let spi: Mutex<NoopRawMutex, _> = Mutex::new(spi);
    let spi = make_static!(spi);

    // Display

    let dc = io.pins.gpio7.into_push_pull_output();
    let rst = io.pins.gpio8.into_push_pull_output();
    let lcd_cs = io.pins.gpio9.into_push_pull_output();
    let spi_dev: SpiDevice<
        '_,
        NoopRawMutex,
        _,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 9>,
    > = SpiDevice::new(spi, lcd_cs);

    let width = 160;
    let height = 80;

    println!("lcd init...");
    let display: ST7735<
        SpiDevice<
            '_,
            NoopRawMutex,
            SpiDma<'static, SPI2, Channel0, FullDuplexMode>,
            GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 9>,
        >,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 7>,
        GpioPin<hal::gpio::Output<hal::gpio::PushPull>, 8>,
    > = ST7735::new(
        spi_dev,
        dc,
        rst,
        st7735::Config {
            rgb: false,
            inverted: false,
            orientation: st7735::Orientation::Landscape,
        },
        width,
        height,
    );
    let display = make_static!(display);

    spawner.spawn(display::init_display(display)).ok();
    spawner.spawn(blink(blink_led)).ok();
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(&stack)).ok();
    spawner.spawn(get_ip_addr(&stack)).ok();
    spawner.spawn(receiving_net_speed(&stack)).ok();

    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
