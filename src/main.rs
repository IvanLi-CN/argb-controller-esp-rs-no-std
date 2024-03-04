#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_backtrace as _;
use esp_wifi::wifi::WifiStaDevice;
use esp_wifi::{initialize, EspWifiInitFor};
use hal::{
    clock::{self, ClockControl},
    embassy,
    gpio::{GpioPin, Unknown},
    peripheral::Peripheral,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rng, IO,
};
use static_cell::make_static;

use embassy_net::{Config, Stack, StackResources};
mod openwrt;
mod wifi;
mod openwrt_types;
use openwrt::netdata_info;
use wifi::{connection, get_ip_addr, net_task};

use esp_backtrace as _;
#[embassy_executor::task]
async fn blink(blink_led: &'static mut GpioPin<Unknown, 1>) {
    let mut blink_led = unsafe { blink_led.clone_unchecked() }.into_push_pull_output();
    blink_led.set_high().unwrap();

    loop {
        blink_led.toggle().unwrap();
        Timer::after(Duration::from_millis(1_00)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks: clock::Clocks<'static> = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio0.into_push_pull_output();
    led.set_high().unwrap();

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

    spawner.spawn(blink(blink_led)).ok();
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(&stack)).ok();
    spawner.spawn(get_ip_addr(&stack)).ok();
    spawner.spawn(netdata_info(&stack)).ok();

    loop {
        led.toggle().unwrap();
        Timer::after(Duration::from_millis(5_00)).await;
        // esp_println::println!("blink!");
    }
}
