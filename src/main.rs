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
use esp_hal::dma::DmaPriority;
use esp_hal::dma::Dma;
use esp_hal::gpio::{Io, Output};
use esp_hal::rng::Rng;
use esp_hal::system::SystemControl;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::{OneShotTimer, PeriodicTimer};
use esp_hal::{
    clock::{self, ClockControl},
    gpio::GpioPin,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
};
use esp_hal::dma_descriptors;
use st7735::ST7735;
use static_cell::make_static;

use embassy_net::{Config, Stack, StackResources};
mod bus;
mod display;
mod udp_client;
mod wifi;
use wifi::{connection, get_ip_addr, net_task};

use esp_backtrace as _;

use crate::udp_client::receiving_net_speed;
#[embassy_executor::task]
async fn blink(blink_led: &'static mut Output<'static, GpioPin<1>>) {
    blink_led.set_high();

    loop {
        blink_led.toggle();
        Timer::after(Duration::from_millis(5_00)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init main");

    // Basic stuff

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks: clock::Clocks<'static> = ClockControl::max(system.clock_control).freeze();
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    let timer0 = OneShotTimer::new(systimer.alarm0.into());
    let timers = [timer0];
    let timers = make_static!(timers);
    esp_hal_embassy::init(&clocks, timers);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let blink_led = Output::new(io.pins.gpio1, esp_hal::gpio::Level::High);

    let blink_led: &'static mut Output<'static, GpioPin<1>> = make_static!(blink_led);

    let timer = PeriodicTimer::new(
        TimerGroup::new(peripherals.TIMG0, &clocks, None)
            .timer0
            .into(),
    );

    // Wi-Fi

    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
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

    let miso = io.pins.gpio4;
    let sda = io.pins.gpio5;
    let sck = io.pins.gpio6;
    let (tx_descriptors, rx_descriptors) = dma_descriptors!(32000, 4096);

    let spi =
        Spi::new(peripherals.SPI2, 40u32.MHz(), SpiMode::Mode0, &clocks)
            .with_sck(sck)
            .with_mosi(sda)
            .with_miso(miso)
            .with_dma(
                dma_channel
                    .configure_for_async(false, DmaPriority::Priority0),
                tx_descriptors,
                rx_descriptors,
            );
    let spi: Mutex<NoopRawMutex, _> = Mutex::new(spi);
    let spi = make_static!(spi);

    // Display

    let dc = Output::new(io.pins.gpio7, esp_hal::gpio::Level::High);
    let rst = Output::new(io.pins.gpio8, esp_hal::gpio::Level::High);
    let lcd_cs = Output::new(io.pins.gpio9, esp_hal::gpio::Level::High);
    let spi_dev: SpiDevice<'_, NoopRawMutex, _, Output<GpioPin<9>>> = SpiDevice::new(spi, lcd_cs);

    let width = 160;
    let height = 80;

    println!("lcd init...");
    let display = ST7735::new(
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
