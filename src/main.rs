#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_backtrace as _;
use hal::gpio::Unknown;
use hal::peripheral::Peripheral;
use hal::{
    clock::ClockControl,
    embassy,
    gpio::GpioPin,
    peripherals::Peripherals,
    prelude::*,
    IO,
};
use static_cell::make_static;

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
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio0.into_push_pull_output();
    led.set_high().unwrap();

    let blink_led: &'static mut GpioPin<Unknown, 1> = make_static!(io.pins.gpio1);

    embassy::init(
        &clocks,
        hal::timer::TimerGroup::new(peripherals.TIMG0, &clocks),
    );

    spawner.spawn(blink(blink_led)).ok();

    loop {
        led.toggle().unwrap();
        Timer::after(Duration::from_millis(5_00)).await;
    }
}
