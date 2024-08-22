#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(const_refs_to_static)]
#![feature(const_maybe_uninit_write)]
#![feature(const_mut_refs)]
#![feature(int_roundings)]

use core::ops::Div;

use argb::ARGBError;
use argb::ARGB;
use embassy_executor::Spawner;
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::Ticker;
use esp_backtrace as _;
use esp_hal::dma::Dma;
use esp_hal::gpio::Io;
use esp_hal::rmt::Rmt;
use esp_hal::rmt::TxChannel;
use esp_hal::rmt::TxChannelConfig;
use esp_hal::rmt::TxChannelCreator;
use esp_hal::rng::Rng;
use esp_hal::system::SystemControl;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::{OneShotTimer, PeriodicTimer};
use esp_hal::{
    clock::{self, ClockControl},
    peripherals::Peripherals,
    prelude::*,
};
use esp_println::println;
use esp_wifi::wifi::WifiStaDevice;
use esp_wifi::{initialize, EspWifiInitFor};
use palette::Hsl;
use palette::SetHue;
use palette::Srgb;
use static_cell::make_static;

use embassy_net::{Config, Stack, StackResources};
mod argb;
mod bus;
mod udp_client;
mod wifi;
use palette::IntoColor;
use wifi::{connection, get_ip_addr, net_task};

use esp_backtrace as _;

#[main]
async fn main(spawner: Spawner) {
    // Basic stuff

    let peripherals = Peripherals::take();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let rmt_pin = io.pins.gpio4;

    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks: clock::Clocks<'static> = ClockControl::max(system.clock_control).freeze();
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    let timer0 = OneShotTimer::new(systimer.alarm0.into());
    let timers = [timer0];
    let timers = make_static!(timers);
    esp_hal_embassy::init(&clocks, timers);

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

    // RMT

    let rmt = Rmt::new(peripherals.RMT, 80.MHz(), &clocks, None).unwrap();

    let channel = rmt
        .channel0
        .configure(
            rmt_pin,
            TxChannelConfig {
                clk_divider: 1,
                idle_output_level: false,
                carrier_modulation: false,
                idle_output: true,

                ..TxChannelConfig::default()
            },
        )
        .unwrap();

    let leds_num = 60;
    let mut buffer = [0u32; 60 * 4 * 8 + 1];

    let mut argb = ARGB::new(channel, &clocks);

    let mut ticker = Ticker::every(Duration::from_millis(16));

    let step_len = 360f64.div(leds_num as f64);
    let mut h_offset = 0f64;

    loop {
        h_offset = (h_offset + 1f64) % 360f64;

        fill_colors(&argb, &mut buffer, h_offset, step_len).unwrap();


        match argb.send(&buffer) {
            Ok(_) => {
                println!("RMT success");
            }
            Err(err) => {
                println!("RMT error: {:?}", err);
            }
        }

        ticker.next().await;
    }
}

fn fill_colors<TX: TxChannel>(
    argb: &ARGB<TX>,
    buffer: &mut [u32],
    h_offset: f64,
    step_len: f64,
) -> Result<(), ARGBError> {
    let mut iter = buffer.iter_mut();

    let mut color = Hsl::new(h_offset, 1.0, 0.5);

    for led_index in 0..60 {
        color.set_hue((led_index as f64) * step_len + h_offset);

        let rgb: Srgb<f64> = color.into_color();

        argb.convert_rgb_channel_to_pulses((rgb.red * 255f64) as u8, &mut iter)
            .unwrap();
        argb.convert_rgb_channel_to_pulses((rgb.green * 255f64) as u8, &mut iter)
            .unwrap();
        argb.convert_rgb_channel_to_pulses((rgb.blue * 255f64) as u8, &mut iter)
            .unwrap();
        argb
            .convert_rgb_channel_to_pulses(128, &mut iter)
            .unwrap();
    }

    *iter.next().ok_or(ARGBError::BufferSizeExceeded)? = 0;

    Ok(())
}
