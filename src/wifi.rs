
use embassy_net::{tcp::TcpSocket, Ipv4Address, Stack, StaticConfigV4};

use embassy_time::{Duration, Timer};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState};
use spin::{Lazy, Mutex};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

// global variable ip address
pub static NETWORK_CONFIG: Lazy<Mutex<Option<StaticConfigV4>>> = Lazy::new(|| {
    println!("initializing");
    Mutex::new(None)
});

#[embassy_executor::task]
pub async fn wifi_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        if  NETWORK_CONFIG.lock().is_none() {
            println!("Waiting for network config...");
            Timer::after(Duration::from_millis(1_000)).await;
            continue;
        }

        Timer::after(Duration::from_millis(1_000)).await;

        let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Address::new(142, 250, 185, 115), 80);
        println!("connecting...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            println!("connect error: {:?}", e);
            continue;
        }
        println!("connected!");
        let mut buf = [0; 1024];
        loop {
            use embedded_io_async::Write;
            let r = socket
                .write_all(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
                .await;
            if let Err(e) = r {
                println!("write error: {:?}", e);
                break;
            }
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    println!("read error: {:?}", e);
                    break;
                }
            };
            println!("{}", core::str::from_utf8(&buf[..n]).unwrap());
        }
        Timer::after(Duration::from_millis(3000)).await;
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
pub async fn get_ip_addr(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    loop {
        if stack.is_link_up() && NETWORK_CONFIG.lock().is_none() {
            println!("Waiting to get IP address...");
            loop {
                if let Some(config) = stack.config_v4() {
                    println!("Got IP: {}", config.address);
                    let mut global_config = NETWORK_CONFIG.lock();
                    *global_config = Some(config);
                    break;
                }
                Timer::after(Duration::from_millis(500)).await;
            }
        }

        if !stack.is_link_up() && NETWORK_CONFIG.lock().is_some() {
            println!("Link down or config down, reset NETWORK_CONFIG to None");
            let mut global_config = NETWORK_CONFIG.lock();
            *global_config = None;
        }

        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}